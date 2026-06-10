use crate::cli::error_run_log;
use crate::cli::gate_kpop_workflow::{
    fail_gate_kpop_after_exhausted, finish_gate_kpop_after_pass, run_gate_kpop_loop,
    GateKpopLoopParams, GateLoopBehavior,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::workflow_kpop_shared::{gate_kpop_loop_iterations, gate_kpop_session_declared_solved};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::{prepare_revise_kpop_run, ReviseKpopPrepared};
use super::{effective_revise_max_loops, ReviseArgs};

struct ReviseFinishInput<'a> {
    shared: &'a SharedOpts,
    prepared: &'a ReviseKpopPrepared,
    agent_ran: bool,
    run_timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    iterations: usize,
}

pub(crate) fn validate_revise_output(resolved_doc_path: &std::path::Path) -> Result<(), String> {
    let meta = std::fs::metadata(resolved_doc_path).map_err(|_| {
        format!(
            "malvin revise: expected document at `{}`",
            resolved_doc_path.display()
        )
    })?;
    if !meta.is_file() || meta.len() == 0 {
        return Err(format!(
            "malvin revise: expected non-empty document at `{}`",
            resolved_doc_path.display()
        ));
    }
    Ok(())
}

fn finish_revise_after_session(input: &ReviseFinishInput<'_>) -> Result<(), String> {
    if !gate_kpop_session_declared_solved(&input.prepared.inner.artifacts, input.iterations)? {
        return Err(
            "malvin revise: agent did not declare KPOP_SOLVED in the session exp log".to_string(),
        );
    }
    validate_revise_output(&input.prepared.resolved_doc_path)?;
    finish_gate_kpop_after_pass(
        input.shared,
        &input.prepared.inner,
        input.agent_ran,
        input.run_timing,
    )
}

fn resolve_revise_gate_outcome(
    input: &ReviseFinishInput<'_>,
    last_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), String> {
    finish_revise_after_session(input).or_else(|e| {
        if input.agent_ran {
            Err(e)
        } else {
            fail_gate_kpop_after_exhausted(
                "malvin revise",
                &input.prepared.inner,
                last_backups,
                GateLoopBehavior::REVISE.restore_malvin_checks_after_session(),
            )
        }
    })
}

pub async fn run_revise(
    revise: ReviseArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let prepared = prepare_revise_kpop_run(&revise.doc_path, workflow)?;
    error_run_log::set_command_error_run_dir(Some(prepared.inner.artifacts.run_dir.clone()));

    emit_run_startup_sequence(
        &prepared.inner.artifacts,
        RunStartupEmitOpts {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources: true,
        },
        &prepared.inner.startup_emit_request,
    )?;

    let max_loops = effective_revise_max_loops(revise.max_loops);
    let max_hypotheses = revise.max_hypotheses.max(1);
    let iterations = gate_kpop_loop_iterations(max_loops);
    let (_gates_ok, agent_ran, run_timing, last_backups) = run_gate_kpop_loop(GateKpopLoopParams {
        shared,
        workflow,
        prepared: &prepared.inner,
        max_loops,
        max_hypotheses,
        behavior: GateLoopBehavior::REVISE,
    })
    .await?;

    let summarize_res = crate::cli::kpop_summarize::run_outer_loop_summarize_if_warranted(
        &crate::cli::kpop_summarize::OuterLoopSummarizeParams {
            max_loops,
            agent_ran,
            shared,
            workflow,
            store: prepared.inner.store(),
            artifacts: prepared.inner.artifacts(),
            malvin_command: "malvin revise",
        },
    )
    .await;

    let finish_input = ReviseFinishInput {
        shared,
        prepared: &prepared,
        agent_ran,
        run_timing: run_timing.as_ref(),
        iterations,
    };
    let gate_r = resolve_revise_gate_outcome(&finish_input, &last_backups);
    let r = crate::cli::kpop_summarize::prefer_gate_outcome_over_summarize(gate_r, summarize_res);

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
    fn revise_post_session_validates_output_file_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let missing = tmp.path().join("doc.md");
        let err = validate_revise_output(&missing).expect_err("missing");
        assert!(err.contains("expected document"));
    }

    #[test]
    fn revise_post_session_validates_output_file_non_empty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let empty = tmp.path().join("doc.md");
        std::fs::write(&empty, "").expect("write");
        let err = validate_revise_output(&empty).expect_err("empty");
        assert!(err.contains("non-empty"));
    }

    #[test]
    fn revise_post_session_accepts_non_empty_document() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let doc = tmp.path().join("doc.md");
        std::fs::write(&doc, "# Revised\n\nClear prose.\n").expect("write");
        validate_revise_output(&doc).expect("ok");
    }

    #[test]
    fn revise_session_succeeded_reads_marker() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = create_kpop_run_artifacts("revise", Some(tmp.path())).expect("artifacts");
        let exp = artifacts.gate_exp_log_path(1);
        std::fs::create_dir_all(exp.parent().unwrap()).expect("mkdir");
        std::fs::write(&exp, "## KPOP_SOLVED\n").expect("write");
        assert!(gate_kpop_session_declared_solved(&artifacts, 1).expect("read"));
    }

    #[test]
    fn revise_run_loop_entry_is_covered() {
        let _ = run_revise;
    }
}
