use crate::kpop_turn_prompts::KpopTurnPrompts;

use crate::agent_backend::agent_backend_timing;

use crate::acp::{
    backoff_after_agent_failure, kpop_fail_after_prompt, restore_session_dotfiles, AgentError,
    CoderPromptOptions, KpopFailAfterPrompt,
};
use crate::cli::workflow_kpop_shared::{
    finish_kpop_acp_session, gate_iteration_context, post_kpop_session_gates,
};
use crate::cli::SharedOpts;
use crate::run_timing::TimingPhase;

use super::params::GateKpopIterationParams;
use super::prepared::GateKpopPrepared;

pub(crate) struct GateKpopMultiturnCtx<'a> {
    pub iteration: &'a mut GateKpopIterationParams<'a>,
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

async fn run_gate_kpop_single_turn(ctx: &mut GateKpopMultiturnCtx<'_>) -> Result<(), String> {
    let prepared = ctx.iteration.loop_params.prepared;
    let prompt = build_gate_kpop_prompt(ctx)?;
    let work_dir = prepared.artifacts().work_dir.as_path();
    let log_path = prepared.artifacts().log_path("kpop");
    ctx.iteration
        .client
        .begin_coder_session(work_dir)
        .await
        .map_err(|e| e.to_string())?;
    let prompt_result = ctx
        .iteration
        .client
        .run_coder_prompt(
            &prompt,
            log_path.as_path(),
            "kpop",
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                single_attempt: true,
                ..Default::default()
            },
        )
        .await;
    finalize_gate_kpop_turn(ctx, work_dir, prompt_result).await
}

pub(crate) async fn run_gate_kpop_session(ctx: &mut GateKpopMultiturnCtx<'_>) -> Result<(), String> {
    let max_attempts = ctx.iteration.client.max_acp_retries();
    let mut last_error = String::new();
    let mut attempts_used = 0_u32;
    for attempt in 1..=max_attempts {
        attempts_used = attempt;
        match run_gate_kpop_single_turn(ctx).await {
            Ok(()) => {
                return finish_kpop_acp_session(
                    ctx.iteration.loop_params.prepared.artifacts(),
                    ctx.iteration.session_dotfile_backups,
                    ctx.iteration
                        .loop_params
                        .behavior
                        .restore_malvin_checks_after_session(),
                )
                .await;
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

pub(crate) fn finish_gate_kpop_after_pass(
    _shared: &SharedOpts,
    prepared: &GateKpopPrepared,
    _agent_ran: bool,
    run_timing: Option<&std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
) -> Result<(), String> {
    if let Some(timing) = run_timing {
        crate::run_timing::finalize_and_emit_run_timing(&prepared.artifacts().run_dir, timing)
            .map_err(|e| e.to_string())?;
    } else {
        crate::run_timing::print_summary_from_run_dir(&prepared.artifacts().run_dir)
            .map_err(|e| e.to_string())?;
    }
    crate::agent_phase::print_done_with_reporting_phase();
    Ok(())
}

pub(crate) fn fail_gate_kpop_after_exhausted(
    command: &str,
    prepared: &GateKpopPrepared,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
    behavior: super::behavior::GateLoopBehavior,
) -> Result<(), String> {
    post_gate_kpop_gates(command, prepared, session_dotfile_backups, behavior)
}

#[cfg(test)]
mod tests {
    #[test]
    fn gate_kpop_session_declared_solved_detects_kpop_solved_marker() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("exp.md");
        std::fs::write(&path, "## KPOP_SOLVED\n").expect("write");
        assert!(super::super::run_loop::session_wrote_kpop_solved(&path).expect("read"));
    }
}
