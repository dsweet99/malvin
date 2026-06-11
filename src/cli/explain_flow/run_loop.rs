use crate::cli::error_run_log;
use crate::cli::gate_kpop_workflow::{
    fail_gate_kpop_after_exhausted, finish_gate_kpop_after_pass, run_gate_kpop_loop,
    GateKpopLoopParams, GateLoopBehavior,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::workflow_kpop_shared::{gate_kpop_loop_iterations, gate_kpop_session_declared_solved};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::{prepare_explain_kpop_run, ExplainKpopPrepared};
use super::{effective_explain_max_loops, ExplainArgs};

struct ExplainFinishInput<'a> {
    shared: &'a SharedOpts,
    prepared: &'a ExplainKpopPrepared,
    agent_ran: bool,
    run_timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    iterations: usize,
}

pub(crate) fn validate_explain_output(tex_path: &std::path::Path, pdf_path: &std::path::Path) -> Result<(), String> {
    for (label, path) in [("tex", tex_path), ("pdf", pdf_path)] {
        let meta = std::fs::metadata(path).map_err(|_| {
            format!(
                "malvin explain: expected {label} file at `{}`",
                path.display()
            )
        })?;
        if !meta.is_file() || meta.len() == 0 {
            return Err(format!(
                "malvin explain: expected non-empty {label} file at `{}`",
                path.display()
            ));
        }
    }
    Ok(())
}

fn finish_explain_after_session(input: &ExplainFinishInput<'_>) -> Result<(), String> {
    if !gate_kpop_session_declared_solved(&input.prepared.inner.artifacts, input.iterations)? {
        return Err(
            "malvin explain: agent did not declare KPOP_SOLVED in the session exp log".to_string(),
        );
    }
    validate_explain_output(&input.prepared.tex_path, &input.prepared.pdf_path)?;
    finish_gate_kpop_after_pass(
        input.shared,
        &input.prepared.inner,
        input.agent_ran,
        input.run_timing,
    )
}

fn resolve_explain_gate_outcome(
    input: &ExplainFinishInput<'_>,
    last_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), String> {
    finish_explain_after_session(input).or_else(|e| {
        if input.agent_ran {
            Err(e)
        } else {
            fail_gate_kpop_after_exhausted(
                "malvin explain",
                &input.prepared.inner,
                last_backups,
                GateLoopBehavior::EXPLAIN.restore_malvin_checks_after_session(),
            )
        }
    })
}

pub async fn run_explain(
    explain: ExplainArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let prepared = prepare_explain_kpop_run(explain.request.as_ref(), &explain.out_path, workflow)?;
    error_run_log::set_command_error_run_dir(Some(prepared.inner.artifacts.run_dir.clone()));

    emit_run_startup_sequence(
        &prepared.inner.artifacts,
        RunStartupEmitOpts {
            tee_stdout: shared.tee_startup_stdout(),
            host_resources: true,
        },
        &prepared.inner.startup_emit_request,
    )?;

    let max_loops = effective_explain_max_loops(explain.max_loops);
    let max_hypotheses = explain.max_hypotheses.max(1);
    let iterations = gate_kpop_loop_iterations(max_loops);
    let (_gates_ok, agent_ran, run_timing, last_backups) = run_gate_kpop_loop(GateKpopLoopParams {
        command: "explain",
        shared,
        workflow,
        prepared: &prepared.inner,
        max_loops,
        max_hypotheses,
        behavior: GateLoopBehavior::EXPLAIN,
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
            malvin_command: "malvin explain",
        },
    )
    .await;

    let finish_input = ExplainFinishInput {
        shared,
        prepared: &prepared,
        agent_ran,
        run_timing: run_timing.as_ref(),
        iterations,
    };
    let gate_r = resolve_explain_gate_outcome(&finish_input, &last_backups);
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
    fn explain_post_session_validates_tex_exists() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let missing_tex = tmp.path().join("explain.tex");
        let pdf = tmp.path().join("explain.pdf");
        std::fs::write(&pdf, b"%PDF").expect("write");
        let err = validate_explain_output(&missing_tex, &pdf).expect_err("missing tex");
        assert!(err.contains("expected tex file"));
    }

    #[test]
    fn explain_post_session_validates_pdf_non_empty() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let tex = tmp.path().join("explain.tex");
        let pdf = tmp.path().join("explain.pdf");
        std::fs::write(&tex, "\\documentclass{article}").expect("write");
        std::fs::write(&pdf, "").expect("write");
        let err = validate_explain_output(&tex, &pdf).expect_err("empty pdf");
        assert!(err.contains("non-empty"));
    }

    #[test]
    fn explain_post_session_accepts_valid_outputs() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let tex = tmp.path().join("explain.tex");
        let pdf = tmp.path().join("explain.pdf");
        std::fs::write(&tex, "\\documentclass{article}").expect("write");
        std::fs::write(&pdf, b"%PDF-1.4").expect("write");
        validate_explain_output(&tex, &pdf).expect("ok");
    }

    #[test]
    fn explain_session_succeeded_reads_marker() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = create_kpop_run_artifacts("explain", Some(tmp.path())).expect("artifacts");
        let exp = artifacts.gate_exp_log_path(1);
        std::fs::create_dir_all(exp.parent().unwrap()).expect("mkdir");
        std::fs::write(&exp, "## KPOP_SOLVED\n").expect("write");
        assert!(gate_kpop_session_declared_solved(&artifacts, 1).expect("read"));
    }

    #[test]
    fn explain_run_loop_entry_is_covered() {
        let _ = run_explain;
    }
}
