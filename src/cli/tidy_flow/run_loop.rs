use crate::cli::error_run_log;
use crate::cli::gate_kpop_workflow::{
    fail_gate_kpop_after_exhausted, finish_gate_kpop_after_pass, run_gate_kpop_loop,
    GateKpopLoopParams, GateLoopBehavior,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::prepare_tidy_kpop_run;
use super::{effective_tidy_max_loops, TidyArgs};

pub async fn run_tidy(
    tidy: TidyArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let prepared = prepare_tidy_kpop_run(workflow)?;
    error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));

    emit_run_startup_sequence(
        &prepared.artifacts,
        RunStartupEmitOpts {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources: true,
        },
        &prepared.startup_emit_request,
    )?;

    let max_loops = effective_tidy_max_loops(tidy.max_loops);
    let max_hypotheses = tidy.max_hypotheses.max(1);
    let (gates_ok, agent_ran, run_timing) = run_gate_kpop_loop(GateKpopLoopParams {
        shared,
        workflow,
        prepared: &prepared,
        max_loops,
        max_hypotheses,
        behavior: GateLoopBehavior::TIDY,
    })
    .await?;

    let r = if gates_ok {
        finish_gate_kpop_after_pass(shared, &prepared, agent_ran, run_timing.as_ref())
    } else {
        fail_gate_kpop_after_exhausted("malvin tidy", &prepared)
    };

    if r.is_ok() {
        error_run_log::clear_command_error_run_dir();
    }
    let _ = &prepared.malvin_checks_backup;
    r
}

#[cfg(test)]
mod tests {
    #[test]
    fn tidy_run_loop_entry_is_covered() {
        let _ = super::run_tidy;
    }
}
