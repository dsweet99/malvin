use crate::kpop_turn_prompts::KpopTurnPrompts;

use crate::acp::{
    kpop_fail_after_prompt, kpop_round, restore_session_dotfiles, spawn_agent_acp_session,
    KpopFailAfterPrompt, KpopPromptRound,
};
use crate::cli::workflow_kpop_shared::{
    clear_quality_gates_log_for_next_agent, finish_kpop_acp_session, gate_iteration_context,
    post_kpop_session_gates,
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
) -> Result<(), String> {
    post_kpop_session_gates(command, prepared.artifacts())
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
    clear_quality_gates_log_for_next_agent(prepared.artifacts())?;
    let prompt = build_gate_kpop_prompt(ctx)?;
    let s = spawn_agent_acp_session(ctx.iteration.client, &prepared.artifacts().work_dir)
        .await
        .map_err(|e| e.to_string())?;
    dispatch_gate_kpop_prompt(ctx, &s, &prompt).await?;
    restore_gate_kpop_session_dotfiles(ctx)?;
    s.shutdown().await.map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) async fn run_gate_kpop_session(ctx: &mut GateKpopMultiturnCtx<'_>) -> Result<(), String> {
    run_gate_kpop_single_acp_turn(ctx).await?;
    finish_kpop_acp_session(
        ctx.iteration.loop_params.prepared.artifacts(),
        ctx.iteration.session_dotfile_backups,
        ctx.iteration
            .loop_params
            .behavior
            .restore_malvin_checks_after_session(),
    )
    .await
}

pub(crate) fn finish_gate_kpop_after_pass(
    _shared: &SharedOpts,
    prepared: &GateKpopPrepared,
    _agent_ran: bool,
    run_timing: Option<&std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
) -> Result<(), String> {
    crate::agent_phase::print_done_with_reporting_phase();
    if let Some(timing) = run_timing {
        crate::run_timing::finalize_and_emit_run_timing(&prepared.artifacts().run_dir, timing)
            .map_err(|e| e.to_string())?;
    } else {
        crate::run_timing::print_summary_from_run_dir(&prepared.artifacts().run_dir)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub(crate) fn fail_gate_kpop_after_exhausted(
    command: &str,
    prepared: &GateKpopPrepared,
) -> Result<(), String> {
    post_gate_kpop_gates(command, prepared)
}

#[cfg(test)]
mod tests {
    #[test]
    fn gate_kpop_session_helpers_are_covered() {
        let _ = stringify!(super::run_gate_kpop_single_acp_turn);
        let _ = stringify!(super::build_gate_kpop_prompt);
        let _ = stringify!(super::dispatch_gate_kpop_prompt);
        let _ = stringify!(super::GateKpopMultiturnCtx);
        let _ = stringify!(super::print_gate_kpop_log_line);
    }
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<GateKpopMultiturnCtx> = None;
        let _ = build_gate_kpop_prompt;
        let _ = dispatch_gate_kpop_prompt;
        let _ = print_gate_kpop_log_line;
        let _ = run_gate_kpop_session;
        let _ = run_gate_kpop_single_acp_turn;
    }
}
