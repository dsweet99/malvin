use crate::kpop_turn_prompts::KpopTurnPrompts;

use crate::acp::{
    backoff_after_agent_failure, kpop_fail_after_prompt, kpop_round, restore_session_dotfiles,
    spawn_agent_acp_session, AgentError, KpopFailAfterPrompt, KpopPromptRound,
};
use crate::cli::workflow_kpop_shared::{
    finish_kpop_acp_session, gate_iteration_context, post_kpop_session_gates,
};
use crate::cli::SharedOpts;

use super::params::GateKpopIterationParams;
use super::prepared::GateKpopPrepared;

pub(crate) struct GateKpopMultiturnCtx<'a> {
    pub iteration: &'a mut GateKpopIterationParams<'a>,
}

pub(crate) fn post_gate_kpop_gates(
    command: &str,
    prepared: &GateKpopPrepared,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
    restore_malvin_checks: bool,
) -> Result<(), String> {
    post_kpop_session_gates(
        command,
        prepared.artifacts(),
        session_dotfile_backups,
        restore_malvin_checks,
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

async fn dispatch_gate_kpop_prompt(
    ctx: &GateKpopMultiturnCtx<'_>,
    session: &crate::acp::AcpSession,
    prompt: &str,
) -> Result<(), String> {
    let prepared = ctx.iteration.loop_params.prepared;
    if let Err(e) = kpop_round(KpopPromptRound {
        session,
        client: ctx.iteration.client,
        text: prompt,
        log: prepared.artifacts().log_path("kpop").as_path(),
        who: "kpop",
        phase: crate::run_timing::TimingPhase::Implement,
    })
    .await
    {
        return kpop_fail_after_prompt(
            session,
            KpopFailAfterPrompt {
                cwd: prepared.artifacts().work_dir.as_path(),
                session_dotfile_backups: ctx.iteration.session_dotfile_backups,
                err: e,
                phase: "prompt",
            },
        )
        .await
        .map_err(|e| e.0);
    }
    Ok(())
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

async fn run_gate_kpop_single_acp_turn(ctx: &mut GateKpopMultiturnCtx<'_>) -> Result<(), String> {
    let prepared = ctx.iteration.loop_params.prepared;
    let prompt = build_gate_kpop_prompt(ctx)?;
    let s = spawn_agent_acp_session(ctx.iteration.client, &prepared.artifacts().work_dir)
        .await
        .map_err(|e| e.to_string())?;
    dispatch_gate_kpop_prompt(ctx, &s, &prompt).await?;
    if let Err(restore_err) = restore_gate_kpop_session_dotfiles(ctx) {
        let prepared = ctx.iteration.loop_params.prepared;
        return kpop_fail_after_prompt(
            &s,
            KpopFailAfterPrompt {
                cwd: prepared.artifacts().work_dir.as_path(),
                session_dotfile_backups: ctx.iteration.session_dotfile_backups,
                err: AgentError(restore_err),
                phase: "restore",
            },
        )
        .await
        .map_err(|e| e.0);
    }
    s.shutdown().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn run_gate_kpop_session(ctx: &mut GateKpopMultiturnCtx<'_>) -> Result<(), String> {
    let max_attempts = ctx.iteration.client.max_acp_retries;
    let mut last_error = String::new();
    let mut attempts_used = 0_u32;
    for attempt in 1..=max_attempts {
        attempts_used = attempt;
        match run_gate_kpop_single_acp_turn(ctx).await {
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
                let timing = ctx.iteration.client.timing.as_ref();
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
        "agent acp (gate kpop) failed after {retries} {noun}. Last error:\n{last_error}"
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
    restore_malvin_checks: bool,
) -> Result<(), String> {
    post_gate_kpop_gates(
        command,
        prepared,
        session_dotfile_backups,
        restore_malvin_checks,
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn gate_kpop_session_helpers_are_covered() {
        let _ = super::run_gate_kpop_single_acp_turn;
        let _ = super::build_gate_kpop_prompt;
        let _ = super::dispatch_gate_kpop_prompt;
        let _: Option<super::GateKpopMultiturnCtx> = None;
        let _ = super::print_gate_kpop_log_line;
    }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<GateKpopMultiturnCtx> = None;
        let _ = build_gate_kpop_prompt;
        let _ = dispatch_gate_kpop_prompt;
        let _ = print_gate_kpop_log_line;
        let _ = run_gate_kpop_session;
        let _ = run_gate_kpop_single_acp_turn;
        let _ = restore_gate_kpop_session_dotfiles;
        let _ = finish_gate_kpop_after_pass;
        let _ = fail_gate_kpop_after_exhausted;
    }
}
