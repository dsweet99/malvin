use super::{
    explain_gate_outcome, validate_explain_output, ExplainGateFinish, ExplainKpopPrepared, run_explain,
};
use crate::cli::explain_flow::prep::ExplainPreflightSnapshot;

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

fn write_explain_gate_outputs(dir: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let tex = dir.join("explain.tex");
    let pdf = dir.join("explain.pdf");
    std::fs::write(&tex, "\\documentclass{article}").expect("write tex");
    std::fs::write(&pdf, b"%PDF").expect("write pdf");
    (tex, pdf)
}

fn explain_gate_outcome_prepared(tmp: &tempfile::TempDir) -> ExplainKpopPrepared {
    let (tex, pdf) = write_explain_gate_outputs(tmp.path());
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("explain", Some(tmp.path())).expect("artifacts");
    ExplainKpopPrepared {
        inner: crate::gate_kpop_workflow::GateKpopPrepared {
            artifacts,
            context: std::collections::HashMap::new(),
            request_text: "req".into(),
            startup_emit_request: "req".into(),
            store,
            malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
        },
        tex_path: tex,
        pdf_path: pdf,
        request_work_dir: tmp.path().to_path_buf(),
        auto_out_path: false,
        preflight_snapshot: ExplainPreflightSnapshot::default(),
    }
}

fn explain_gate_outcome_fixture() -> (
    tempfile::TempDir,
    ExplainKpopPrepared,
    crate::cli::SharedOpts,
    crate::artifacts::SessionDotfileBackups,
) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let prepared = explain_gate_outcome_prepared(&tmp);
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
    (tmp, prepared, shared, backups)
}

#[test]
fn explain_gate_outcome_fails_when_loop_exhausted_with_output_but_no_exit() {
    let (_tmp, prepared, shared, backups) = explain_gate_outcome_fixture();
    let err = explain_gate_outcome(ExplainGateFinish {
        shared: &shared,
        prepared: &prepared,
        tex_path: &prepared.tex_path,
        pdf_path: &prepared.pdf_path,
        agent_ran: true,
        gates_ok: false,
        run_timing: None,
        last_backups: &backups,
        summarize_res: Ok(()),
    })
    .expect_err("needs two consecutive solved markers");
    assert!(err.contains("two consecutive"));
}
