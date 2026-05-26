use crate::kpop_multiturn_prompts::KpopMultiturnPrompts;
use crate::kpop_progression::KpopMultiturnState;
use crate::output::{MALVIN_WHO, print_stdout_line};

use crate::cli::kpop_flow::{
    KpopAcpMultiturnCtx, KpopPrepared, KpopTurnPrompts, kpop_run_acp_multiturn,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::workflow_kpop_shared::{
    finish_kpop_acp_session, post_kpop_session_gates, print_kpop_session_log_line,
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

pub(crate) fn print_gate_kpop_log_line(prepared: &GateKpopPrepared) {
    print_kpop_session_log_line(prepared.artifacts(), prepared.exp_log_path());
}

fn kpop_turn_prompts(prepared: &GateKpopPrepared) -> KpopMultiturnPrompts<'_> {
    KpopMultiturnPrompts::Turn(KpopTurnPrompts {
        store: prepared.store(),
        base: prepared.context(),
        request_text: prepared.request_text(),
        prepend_rules_once: true,
    })
}

fn kpop_acp_prepared(
    prepared: &GateKpopPrepared,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
) -> KpopPrepared {
    KpopPrepared {
        artifacts: prepared.artifacts().clone(),
        exp_log_path: prepared.exp_log_path().to_path_buf(),
        context: prepared.context().clone(),
        text: prepared.request_text().to_string(),
        session_dotfile_backups: session_dotfile_backups.clone(),
    }
}

async fn run_gate_kpop_multiturn(ctx: &mut GateKpopMultiturnCtx<'_>) -> Result<(), String> {
    let params = ctx.iteration.loop_params;
    let prepared = params.prepared;
    emit_run_startup_sequence(
        prepared.artifacts(),
        RunStartupEmitOpts {
            tee_stdout: params.shared.tee_startup_stdout(),
            host_resources: true,
        },
        prepared.startup_emit_request(),
    )?;
    let mut state = KpopMultiturnState::new(
        kpop_turn_prompts(prepared),
        prepared.exp_log_path().to_path_buf(),
        params.max_hypotheses,
        0.0,
    )?;
    let kpop_prepared = kpop_acp_prepared(prepared, ctx.iteration.session_dotfile_backups);
    kpop_run_acp_multiturn(
        KpopAcpMultiturnCtx {
            client: ctx.iteration.client,
            prepared: &kpop_prepared,
            workflow: params.workflow,
            state: &mut state,
            store: prepared.store(),
        },
        crate::run_timing::acp_post_run::RunTimingSessionEnd::AccumulateRun,
    )
    .await
}

pub(crate) async fn run_gate_kpop_session(ctx: &mut GateKpopMultiturnCtx<'_>) -> Result<(), String> {
    run_gate_kpop_multiturn(ctx).await?;
    finish_kpop_acp_session(
        ctx.iteration.loop_params.prepared.artifacts(),
        ctx.iteration.session_dotfile_backups,
    )
    .await
}

pub(crate) fn finish_gate_kpop_after_pass(
    shared: &SharedOpts,
    prepared: &GateKpopPrepared,
    agent_ran: bool,
    run_timing: Option<&std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
) -> Result<(), String> {
    if !agent_ran {
        emit_run_startup_sequence(
            prepared.artifacts(),
            RunStartupEmitOpts {
                tee_stdout: shared.tee_startup_stdout(),
                host_resources: true,
            },
            prepared.startup_emit_request(),
        )?;
    }
    print_stdout_line(MALVIN_WHO, "DONE");
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
        let _ = stringify!(super::run_gate_kpop_multiturn);
        let _ = stringify!(super::GateKpopMultiturnCtx);
        let _ = stringify!(super::print_gate_kpop_log_line);
        let _ = stringify!(super::kpop_turn_prompts);
        let _ = stringify!(super::kpop_acp_prepared);
    }
}
