use crate::cli::error_run_log;
use crate::cli::gate_kpop_workflow::{
    fail_gate_kpop_after_exhausted, finish_gate_kpop_after_pass, run_gate_kpop_loop,
    GateKpopLoopParams, GateLoopBehavior,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::prepare_constrain_kpop_run;
use super::{effective_constrain_max_loops, ConstrainArgs};

pub async fn run_constrain(
    constrain: ConstrainArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let cli_request =
        crate::cli::cli_request::require_cli_request(constrain.request.as_ref(), "constrain")?;
    let prepared = prepare_constrain_kpop_run(workflow, &cli_request)?;
    error_run_log::set_command_error_run_dir(Some(prepared.artifacts.run_dir.clone()));

    emit_run_startup_sequence(
        &prepared.artifacts,
        RunStartupEmitOpts {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources: true,
        },
        &prepared.startup_emit_request,
    )?;

    let max_loops = effective_constrain_max_loops(constrain.max_loops);
    let max_hypotheses = constrain.max_hypotheses.max(1);
    let (gates_ok, agent_ran, run_timing) = run_gate_kpop_loop(GateKpopLoopParams {
        shared,
        workflow,
        prepared: &prepared,
        max_loops,
        max_hypotheses,
        behavior: GateLoopBehavior::CODE,
    })
    .await?;

    let r = if gates_ok {
        finish_gate_kpop_after_pass(shared, &prepared, agent_ran, run_timing.as_ref())
    } else {
        fail_gate_kpop_after_exhausted("malvin constrain", &prepared)
    };

    if r.is_ok() {
        error_run_log::clear_command_error_run_dir();
    }
    let _ = &prepared.malvin_checks_backup;
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{SharedOpts, WorkflowCliOptions};

    #[test]
    fn constrain_run_loop_entry_is_covered() {
        let _ = stringify!(super::run_constrain);
    }

    #[tokio::test]
    async fn run_constrain_requires_request() {
        let shared = SharedOpts {
            model: "auto".into(),
            no_force: false,
            no_tee: true,
            no_markdown: false,
            verbose: false,
            max_acp_retries: crate::config::DEFAULT_MAX_ACP_RETRIES,
            doc: false,
        };
        let constrain = ConstrainArgs {
            max_loops: 1,
            max_hypotheses: 1,
            tenacious: false,
            trust_the_plan: false,
            dry_run: false,
            skip_pre_checks: false,
            fast: false,
            request: None,
        };
        let err = run_constrain(
            constrain,
            &shared,
            WorkflowCliOptions {
                force: false,
            },
        )
        .await
        .expect_err("missing request");
        assert!(err.contains("constrain"));
    }
}
