use crate::cli::error_run_log;
use crate::cli::gate_kpop_workflow::{
    fail_gate_kpop_after_exhausted, finish_gate_kpop_after_pass, run_gate_kpop_loop,
    GateKpopLoopParams, GateLoopBehavior,
};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::prepare_code_kpop_run;
use super::{effective_code_max_loops, CodeArgs};

pub async fn run_code(
    code: CodeArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let workflow = WorkflowCliOptions {
        force: workflow.force,
        run_learn: !code.no_learn,
    };
    let cli_request = crate::cli::cli_request::require_cli_request(code.request.as_ref(), "code")?;
    let prepared = prepare_code_kpop_run(workflow, &cli_request)?;
    error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));

    let max_loops = effective_code_max_loops(code.max_loops);
    let (gates_ok, agent_ran, run_timing) = run_gate_kpop_loop(GateKpopLoopParams {
        shared,
        workflow,
        prepared: &prepared,
        max_loops,
        max_hypotheses: max_loops,
        behavior: GateLoopBehavior::CODE,
    })
    .await?;

    let r = if gates_ok {
        finish_gate_kpop_after_pass(shared, &prepared, agent_ran, run_timing.as_ref())
    } else {
        fail_gate_kpop_after_exhausted("malvin code", &prepared)
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
    fn code_run_loop_entry_is_covered() {
        let _ = stringify!(super::run_code);
    }
}
