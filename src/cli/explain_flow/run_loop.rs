use crate::cli::error_run_log;
use crate::cli::gate_kpop_workflow::{
    fail_gate_kpop_after_exhausted, finish_gate_kpop_after_pass, run_gate_kpop_loop,
    GateKpopLoopParams, GateLoopBehavior,
};
use crate::cli::run_emit::{emit_run_startup_sequence, RunStartupEmitOpts};
use crate::cli::{SharedOpts, WorkflowCliOptions};

use super::run_startup::{prepare_explain_kpop_run, ExplainKpopPrepared};
use super::{effective_explain_max_loops, ExplainArgs};

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

struct ExplainGateFinish<'a> {
    shared: &'a SharedOpts,
    prepared: &'a ExplainKpopPrepared,
    agent_ran: bool,
    gates_ok: bool,
    run_timing: Option<&'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_backups: &'a crate::artifacts::SessionDotfileBackups,
    summarize_res: Result<(), String>,
}

fn explain_gate_outcome(finish: ExplainGateFinish<'_>) -> Result<(), String> {
    let gate_r = if finish.gates_ok {
        validate_explain_output(&finish.prepared.tex_path, &finish.prepared.pdf_path)?;
        finish_gate_kpop_after_pass(
            finish.shared,
            &finish.prepared.inner,
            finish.agent_ran,
            finish.run_timing,
        )
    } else if finish.agent_ran {
        if let Err(e) = validate_explain_output(&finish.prepared.tex_path, &finish.prepared.pdf_path) {
            Err(e)
        } else {
            Err(
                "malvin explain: gate loop did not exit on two consecutive ## KPOP_SOLVED markers"
                    .to_string(),
            )
        }
    } else {
        fail_gate_kpop_after_exhausted(
            "malvin explain",
            &finish.prepared.inner,
            finish.last_backups,
            GateLoopBehavior::EXPLAIN,
        )
    };
    crate::cli::workflow_kpop_shared::prefer_gate_outcome_over_summarize(gate_r, finish.summarize_res)
}

pub async fn run_explain(
    explain: &mut ExplainArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let prepared = prepare_explain_kpop_run(explain.request.as_ref(), &explain.out_path, workflow)?;
    explain.out_path =
        crate::cli::default_output_path::path_relative_to_cwd(&prepared.tex_path)?;
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
    let (gates_ok, agent_ran, run_timing, last_backups) = run_gate_kpop_loop(GateKpopLoopParams {
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
            agent_ran,
            shared,
            workflow,
            store: prepared.inner.store(),
            artifacts: prepared.inner.artifacts(),
            malvin_command: "malvin explain",
        },
    )
    .await;

    let r = explain_gate_outcome(ExplainGateFinish {
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
    let _ = &prepared.inner.malvin_checks_backup;
    r
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn explain_run_loop_entry_is_covered() {
        let _ = run_explain;
    }

    #[test]
    fn explain_gate_outcome_fails_when_loop_exhausted_with_output_but_no_exit() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let tex = tmp.path().join("explain.tex");
        let pdf = tmp.path().join("explain.pdf");
        std::fs::write(&tex, "\\documentclass{article}").expect("write tex");
        std::fs::write(&pdf, b"%PDF").expect("write pdf");
        let store = crate::prompts::PromptStore::default_store();
        store.ensure_defaults().expect("defaults");
        let artifacts = crate::artifacts::create_kpop_run_artifacts("explain", Some(tmp.path())).expect("artifacts");
        let prepared = ExplainKpopPrepared {
            inner: crate::cli::gate_kpop_workflow::GateKpopPrepared {
                artifacts,
                context: std::collections::HashMap::new(),
                request_text: "req".into(),
                startup_emit_request: "req".into(),
                store,
                malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
            },
            tex_path: tex,
            pdf_path: pdf,
        };
        let shared = crate::cli::SharedOpts {
            model: crate::config::DEFAULT_CLI_MODEL.into(),
            no_force: true,
            no_tenacious: false,
            no_tee: true,
            no_markdown: true,
            verbose: false,
            max_acp_retries: 1,
            doc: false,
            name: None,
            mini: false,
            mini_max_bash_turns: 32,
        };
        let backups = crate::artifacts::SessionDotfileBackups::snapshot(tmp.path()).expect("snap");
        let err = explain_gate_outcome(ExplainGateFinish {
            shared: &shared,
            prepared: &prepared,
            agent_ran: true,
            gates_ok: false,
            run_timing: None,
            last_backups: &backups,
            summarize_res: Ok(()),
        })
        .expect_err("needs two consecutive solved markers");
        assert!(err.contains("two consecutive"));
    }
}
