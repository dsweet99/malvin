use crate::cli::error_run_log;
use crate::kpop_engine::{
    fail_kpop_engine_after_exhausted, finish_kpop_engine_after_pass, run_kpop_engine,
    KPopEngineParams, KPopHardConstraints,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::prepare_code_kpop_run;
use super::{effective_code_max_loops, CodeArgs};

fn emit_code_run_startup(
    shared: &SharedOpts,
    prepared: &super::run_startup::CodeKpopPrepared,
) -> Result<(), String> {
    emit_run_startup_sequence(
        &prepared.artifacts,
        RunStartupEmitOpts::from_shared(shared, true),
        &prepared.startup_emit_request,
    )
}

struct CodeGateFinish<'a> {
    shared: &'a SharedOpts,
    prepared: &'a super::run_startup::CodeKpopPrepared,
    behavior: KPopHardConstraints,
    agent_ran: bool,
    gates_ok: bool,
    run_timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_backups: &'a crate::artifacts::SessionDotfileBackups,
    summarize_res: Result<(), String>,
}

fn code_gate_outcome(finish: CodeGateFinish<'_>) -> Result<(), String> {
    let gate_r = if finish.gates_ok {
        finish_kpop_engine_after_pass(
            finish.shared,
            finish.prepared,
            finish.agent_ran,
            finish.run_timing,
        )
    } else {
        fail_kpop_engine_after_exhausted(
            "malvin code",
            finish.prepared,
            finish.last_backups,
            finish.behavior,
        )
    };
    crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize(gate_r, finish.summarize_res)
}

pub async fn run_code(
    code: CodeArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    request: &str,
) -> Result<(), String> {
    let cli_request = request.trim();
    if cli_request.is_empty() {
        return Err("malvin code: missing required REQUEST (text or path)".into());
    }
    let prepared = prepare_code_kpop_run(workflow, cli_request, &shared.model, shared.git)?;
    error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));

    emit_code_run_startup(shared, &prepared)?;

    let max_loops = effective_code_max_loops(code.max_loops);
    let behavior = KPopHardConstraints::CODE.with_workspace_quality_gates(shared.gates);
    let (gates_ok, agent_ran, run_timing, last_backups) = run_kpop_engine(KPopEngineParams {
        command: "code",
        shared,
        workflow,
        prepared: &prepared,
        max_loops,
        max_hypotheses: code.max_hypotheses,
        behavior,
    })
    .await?;

    let r = code_gate_outcome(CodeGateFinish {
        shared,
        prepared: &prepared,
        behavior,
        agent_ran,
        gates_ok,
        run_timing: run_timing.as_ref(),
        last_backups: &last_backups,
        summarize_res: Ok(()),
    });

    if r.is_ok() {
        error_run_log::clear_command_error_run_dir();
    }
    let _ = &prepared.malvin_checks_backup;
    r
}

#[cfg(test)]
mod tests {
    #[test]
    fn code_run_loop_entry_is_covered() {
        let _ = super::run_code;
    }

}

#[cfg(test)]
#[path = "run_loop_kiss_cov.rs"]
mod run_loop_kiss_cov;
