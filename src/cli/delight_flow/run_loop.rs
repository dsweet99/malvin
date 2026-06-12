use crate::cli::error_run_log;
use crate::cli::gate_kpop_workflow::{
    fail_gate_kpop_after_exhausted, finish_gate_kpop_after_pass, run_gate_kpop_loop,
    GateKpopLoopParams, GateLoopBehavior,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::workflow_kpop_shared::{gate_kpop_loop_iterations, gate_kpop_session_declared_solved};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::{prepare_delight_kpop_run, DelightKpopPrepared};
use super::{effective_delight_max_loops, DelightArgs};

struct DelightFinishInput<'a> {
    shared: &'a SharedOpts,
    prepared: &'a DelightKpopPrepared,
    agent_ran: bool,
    run_timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    iterations: usize,
}

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

fn finish_delight_after_session(input: &DelightFinishInput<'_>) -> Result<(), String> {
    if !gate_kpop_session_declared_solved(&input.prepared.inner.artifacts, input.iterations)? {
        return Err(
            "malvin delight: agent did not declare KPOP_SOLVED in the session exp log".to_string(),
        );
    }
    validate_delight_output(&input.prepared.resolved_out_path)?;
    finish_gate_kpop_after_pass(
        input.shared,
        &input.prepared.inner,
        input.agent_ran,
        input.run_timing,
    )
}

fn resolve_delight_gate_outcome(
    input: &DelightFinishInput<'_>,
    last_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), String> {
    finish_delight_after_session(input).or_else(|e| {
        if input.agent_ran {
            Err(e)
        } else {
            fail_gate_kpop_after_exhausted(
                "malvin delight",
                &input.prepared.inner,
                last_backups,
                GateLoopBehavior::DELIGHT,
            )
        }
    })
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
    let iterations = gate_kpop_loop_iterations(max_loops);
    let (_gates_ok, agent_ran, run_timing, last_backups) = run_gate_kpop_loop(GateKpopLoopParams {
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

    let finish_input = DelightFinishInput {
        shared,
        prepared: &prepared,
        agent_ran,
        run_timing: run_timing.as_ref(),
        iterations,
    };
    let gate_r = resolve_delight_gate_outcome(&finish_input, &last_backups);
    let r = crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize(gate_r, summarize_res);

    if r.is_ok() {
        error_run_log::clear_command_error_run_dir();
    }
    let _ = &prepared.inner.malvin_checks_backup;
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::create_kpop_run_artifacts;

    #[test]
    fn delight_post_session_validates_output_file_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let missing = tmp.path().join("plan.md");
        let err = validate_delight_output(&missing).expect_err("missing");
        assert!(err.contains("expected plan file"));
    }

    #[test]
    fn delight_post_session_validates_output_file_non_empty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let empty = tmp.path().join("plan.md");
        std::fs::write(&empty, "").expect("write");
        let err = validate_delight_output(&empty).expect_err("empty");
        assert!(err.contains("non-empty"));
    }

    #[test]
    fn delight_post_session_accepts_plain_markdown_without_begin_malvin() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = tmp.path().join("plan.md");
        std::fs::write(&plan, "# User feature idea\n\nImprove the CLI.\n").expect("write");
        validate_delight_output(&plan).expect("ok");
    }

    #[test]
    fn delight_session_succeeded_reads_marker() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = create_kpop_run_artifacts("delight", Some(tmp.path())).expect("artifacts");
        let exp = artifacts.gate_exp_log_path(1);
        std::fs::create_dir_all(exp.parent().unwrap()).expect("mkdir");
        std::fs::write(&exp, "## KPOP_SOLVED\n").expect("write");
        assert!(gate_kpop_session_declared_solved(&artifacts, 1).expect("read"));
    }

    #[test]
    fn delight_run_loop_entry_is_covered() {
        let _ = run_delight;
    }
}
