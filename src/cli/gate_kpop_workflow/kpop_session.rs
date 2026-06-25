use crate::kpop_turn_prompts::KpopTurnPrompts;

use crate::agent_backend::agent_backend_timing;

use crate::acp::{
    backoff_after_agent_failure, kpop_fail_after_prompt, restore_session_dotfiles, AgentError,
    CoderPromptOptions, KpopFailAfterPrompt,
};
use crate::cli::workflow_kpop_shared::{
    finish_kpop_acp_session, gate_iteration_context, post_kpop_session_gates,
};
use crate::run_timing::TimingPhase;

use super::params::GateKpopIterationParams;
use super::prepared::GateKpopPrepared;

pub(crate) struct GateKpopMultiturnCtx<'a> {
    pub iteration: &'a mut GateKpopIterationParams<'a>,
}

impl<'a> GateKpopMultiturnCtx<'a> {
    #[cfg(test)]
    pub(crate) const fn iteration_number(&self) -> usize {
        self.iteration.iteration
    }
}

pub(crate) fn post_gate_kpop_gates(
    command: &str,
    prepared: &GateKpopPrepared,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
    behavior: super::behavior::GateLoopBehavior,
) -> Result<(), String> {
    if behavior.skip_workspace_quality_gates {
        return Ok(());
    }
    post_kpop_session_gates(
        command,
        prepared.artifacts(),
        session_dotfile_backups,
        behavior.restore_malvin_checks_after_session(),
    )
}

pub(crate) fn print_gate_kpop_log_line(prepared: &GateKpopPrepared, exp_log_path: &std::path::Path) {
    crate::cli::workflow_kpop_shared::print_kpop_session_log_line(
        prepared.artifacts(),
        exp_log_path,
    );
}

fn build_gate_kpop_prompt(ctx: &GateKpopMultiturnCtx<'_>) -> Result<String, String> {
    let params = ctx.iteration.loop_params;
    let prepared = params.prepared;
    KpopTurnPrompts {
        store: prepared.store(),
        base: &gate_iteration_context(
            prepared.context(),
            prepared.artifacts(),
            &ctx.iteration.exp_log_path,
            ctx.iteration.iteration,
        ),
        request_text: prepared.request_text(),
        prepend_rules_once: false,
    }
    .gate_kpop_single_turn_prompt(params.max_hypotheses)
}

fn restore_gate_kpop_session_dotfiles(ctx: &GateKpopMultiturnCtx<'_>) -> Result<(), String> {
    let prepared = ctx.iteration.loop_params.prepared;
    let work_dir = prepared.artifacts().work_dir.as_path();
    let backups = ctx.iteration.session_dotfile_backups;
    if ctx
        .iteration
        .loop_params
        .behavior
        .restore_malvin_checks_after_session()
    {
        restore_session_dotfiles(work_dir, backups).map_err(|e| e.to_string())
    } else {
        backups.restore_excluding_malvin_checks(work_dir)
    }
}

async fn finalize_gate_kpop_turn(
    ctx: &mut GateKpopMultiturnCtx<'_>,
    work_dir: &std::path::Path,
    prompt_result: Result<(), AgentError>,
) -> Result<(), String> {
    if let Err(restore_err) = restore_gate_kpop_session_dotfiles(ctx) {
        ctx.iteration.client.end_coder_session().await.ok();
        return kpop_fail_after_prompt(KpopFailAfterPrompt {
            cwd: work_dir,
            session_dotfile_backups: ctx.iteration.session_dotfile_backups,
            err: AgentError(restore_err),
            phase: "restore",
        })
        .await
        .map_err(|e| e.0);
    }
    ctx.iteration
        .client
        .end_coder_session()
        .await
        .map_err(|e| e.to_string())?;
    prompt_result.map_err(|e| e.to_string())?;
    Ok(())
}

async fn run_gate_kpop_coder_turn(
    ctx: &mut GateKpopMultiturnCtx<'_>,
    prompt: &str,
    work_dir: &std::path::Path,
    log_path: &std::path::Path,
) -> Result<(), AgentError> {
    let params = ctx.iteration.loop_params;
    let prepared = params.prepared;
    ctx.iteration
        .client
        .begin_coder_session(work_dir)
        .await?;
    let mut prompt_result = ctx
        .iteration
        .client
        .run_coder_prompt(
            prompt,
            log_path,
            "kpop",
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                single_attempt: true,
                ..Default::default()
            },
        )
        .await;
    if prompt_result.is_ok() {
        let malvin_command = format!("malvin {}", params.command);
        prompt_result = crate::cli::kpop_summarize::maybe_run_gate_inline_summarize(
            crate::cli::kpop_summarize::GateInlineSummarizeCtx {
                client: ctx.iteration.client,
                store: prepared.store(),
                artifacts: prepared.artifacts(),
                malvin_command: &malvin_command,
                iteration: ctx.iteration.iteration,
                total_iterations: ctx.iteration.total_iterations,
                consecutive_solved_entering: ctx.iteration.consecutive_solved_entering,
                behavior: params.behavior,
            },
        )
        .await
        .map_err(AgentError);
    }
    prompt_result
}

async fn run_gate_kpop_single_turn(
    ctx: &mut GateKpopMultiturnCtx<'_>,
) -> Result<Option<crate::artifacts::SessionDotfileBackups>, String> {
    let prepared = ctx.iteration.loop_params.prepared;
    let prompt = build_gate_kpop_prompt(ctx)?;
    let work_dir = prepared.artifacts().work_dir.as_path();
    let log_path = prepared.artifacts().log_path("kpop");
    let prompt_result =
        run_gate_kpop_coder_turn(ctx, &prompt, work_dir, log_path.as_path()).await;
    let post_agent_backups = if prompt_result.is_ok() {
        Some(
            crate::artifacts::SessionDotfileBackups::snapshot_after_ensuring_home_config(
                work_dir,
            )?,
        )
    } else {
        None
    };
    finalize_gate_kpop_turn(ctx, work_dir, prompt_result).await?;
    Ok(post_agent_backups)
}

pub(crate) async fn run_gate_kpop_session(
    ctx: &mut GateKpopMultiturnCtx<'_>,
) -> Result<crate::artifacts::SessionDotfileBackups, String> {
    let iteration_start = ctx.iteration.session_dotfile_backups.clone();
    let max_attempts = ctx.iteration.client.max_acp_retries();
    let mut last_error = String::new();
    let mut attempts_used = 0_u32;
    for attempt in 1..=max_attempts {
        attempts_used = attempt;
        match run_gate_kpop_single_turn(ctx).await {
            Ok(post_agent_backups) => {
                finish_kpop_acp_session(
                    ctx.iteration.loop_params.prepared.artifacts(),
                    ctx.iteration.session_dotfile_backups,
                    ctx.iteration
                        .loop_params
                        .behavior
                        .restore_malvin_checks_after_session(),
                )
                .await?;
                let progress = post_agent_backups.unwrap_or_else(|| iteration_start.clone());
                let work_dir = ctx
                    .iteration
                    .loop_params
                    .prepared
                    .artifacts()
                    .work_dir
                    .as_path();
                return Ok(crate::artifacts::merge_and_sanitize_for_gate_restore(
                    &iteration_start,
                    &progress,
                    work_dir,
                ));
            }
            Err(e) => {
                last_error = e;
                let timing = agent_backend_timing(ctx.iteration.client);
                match backoff_after_agent_failure(timing, &last_error, attempt, max_attempts)
                    .await
                {
                    Err(err) => return Err(err.0),
                    Ok(true) => break,
                    Ok(false) => {}
                }
            }
        }
    }
    let retries = attempts_used.saturating_sub(1);
    let noun = crate::acp::retries_noun(retries);
    Err(format!(
        "agent (gate kpop) failed after {retries} {noun}. Last error:\n{last_error}"
    ))
}

#[cfg(test)]
#[path = "kpop_session_kiss_cov_tests.rs"]
mod kpop_session_kiss_cov_tests;

