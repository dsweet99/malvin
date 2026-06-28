use crate::cli::error_run_log;
use crate::kpop_engine::{
    fail_kpop_engine_after_exhausted, finish_kpop_engine_after_pass, run_kpop_engine,
    KPopEngineParams, KPopHardConstraints,
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
    let (gates_ok, agent_ran, run_timing, last_backups) = run_kpop_engine(KPopEngineParams {
        command: "tidy",
        shared,
        workflow,
        prepared: &prepared,
        max_loops,
        max_hypotheses,
        behavior: KPopHardConstraints::TIDY,
    })
    .await?;

    let summarize_res = crate::cli::kpop_summarize::run_outer_loop_summarize_if_warranted(
        &crate::cli::kpop_summarize::OuterLoopSummarizeParams {
            agent_ran,
            shared,
            workflow,
            store: prepared.store(),
            artifacts: prepared.artifacts(),
            malvin_command: "malvin tidy",
        },
    )
    .await;

    let gate_r = if gates_ok {
        finish_kpop_engine_after_pass(shared, &prepared, agent_ran, run_timing.as_ref())
    } else {
        fail_kpop_engine_after_exhausted(
            "malvin tidy",
            &prepared,
            &last_backups,
            KPopHardConstraints::TIDY,
        )
    };
    let r = crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize(gate_r, summarize_res);

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
