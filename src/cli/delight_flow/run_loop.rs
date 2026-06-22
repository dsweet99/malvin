use crate::cli::error_run_log;
use crate::gate_kpop_workflow::{
    fail_gate_kpop_after_exhausted, finish_gate_kpop_after_pass, run_gate_kpop_loop,
    GateKpopLoopParams, GateLoopBehavior,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::{prepare_delight_kpop_run, DelightKpopPrepared};
use super::{effective_delight_max_loops, DelightArgs};

pub(crate) fn validate_delight_output(resolved_out_path: &std::path::Path) -> Result<(), String> {
    let meta = std::fs::metadata(resolved_out_path).map_err(|_| {
        format!(
            "malvin delight: expected plan file at `{}`",
            resolved_out_path.display()
        )
    })?;
    if !meta.is_file() || meta.len() == 0 {
        return Err(format!(
            "malvin delight: expected non-empty plan file at `{}`",
            resolved_out_path.display()
        ));
    }
    Ok(())
}

struct DelightGateFinish<'a> {
    shared: &'a SharedOpts,
    prepared: &'a DelightKpopPrepared,
    agent_ran: bool,
    gates_ok: bool,
    run_timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_backups: &'a crate::artifacts::SessionDotfileBackups,
    summarize_res: Result<(), String>,
}

fn delight_gate_outcome(finish: DelightGateFinish<'_>) -> Result<(), String> {
    let gate_r = if finish.gates_ok {
        validate_delight_output(&finish.prepared.resolved_out_path)?;
        finish_gate_kpop_after_pass(
            finish.shared,
            &finish.prepared.inner,
            finish.agent_ran,
            finish.run_timing,
        )
    } else if finish.agent_ran {
        if let Err(e) = validate_delight_output(&finish.prepared.resolved_out_path) {
            Err(e)
        } else {
            Err(
                "malvin delight: gate loop did not exit on two consecutive ## KPOP_SOLVED markers"
                    .to_string(),
            )
        }
    } else {
        fail_gate_kpop_after_exhausted(
            "malvin delight",
            &finish.prepared.inner,
            finish.last_backups,
            GateLoopBehavior::DELIGHT,
        )
    };
    crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize(gate_r, finish.summarize_res)
}

pub async fn run_delight(
    delight: &mut DelightArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let prepared = prepare_delight_kpop_run(&delight.out_path, delight.guidance.as_ref(), workflow)?;
    delight.out_path =
        crate::cli::default_output_path::path_relative_to_cwd(&prepared.resolved_out_path)?;
    error_run_log::set_command_error_run_dir(Some(prepared.inner.artifacts.run_dir.clone()));

    emit_run_startup_sequence(
        &prepared.inner.artifacts,
        RunStartupEmitOpts {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources: true,
        },
        &prepared.inner.startup_emit_request,
    )?;

    let max_loops = effective_delight_max_loops(delight.max_loops);
    let max_hypotheses = delight.max_hypotheses.max(1);
    let (gates_ok, agent_ran, run_timing, last_backups) = run_gate_kpop_loop(GateKpopLoopParams {
        command: "delight",
        shared,
        workflow,
        prepared: &prepared.inner,
        max_loops,
        max_hypotheses,
        behavior: GateLoopBehavior::DELIGHT,
    })
    .await?;

    let summarize_res = crate::cli::kpop_summarize::run_outer_loop_summarize_if_warranted(
        &crate::cli::kpop_summarize::OuterLoopSummarizeParams {
            agent_ran,
            shared,
            workflow,
            store: prepared.inner.store(),
            artifacts: prepared.inner.artifacts(),
            malvin_command: "malvin delight",
        },
    )
    .await;

    let r = delight_gate_outcome(DelightGateFinish {
        shared,
        prepared: &prepared,
        agent_ran,
        gates_ok,
        run_timing: run_timing.as_ref(),
        last_backups: &last_backups,
        summarize_res,
    });

    if r.is_ok() {
        error_run_log::clear_command_error_run_dir();
    }
    r
}
#[cfg(test)]
#[path = "run_loop_test.rs"]
mod run_loop_test;
#[cfg(test)]
#[path = "run_loop_kiss_cov_test.rs"]
mod run_loop_kiss_cov_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<DelightGateFinish> = None;
    }
}
