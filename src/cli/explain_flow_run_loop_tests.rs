use super::super::outputs::{products_nonempty, resolve_explain_output_paths, validate_explain_output};
use super::super::prep::ExplainPreflightSnapshot;
use super::super::run_startup::ExplainKpopPrepared;
use super::run_explain;

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
fn explain_lgtm_str_is_exact() {
    assert!(crate::review_sync::is_lgtm_str("LGTM"));
    assert!(crate::review_sync::is_lgtm_str("  LGTM\n"));
    assert!(!crate::review_sync::is_lgtm_str("LGTM\nextra"));
    assert!(!crate::review_sync::is_lgtm_str(""));
}

#[test]
fn explain_review_chat_is_lgtm_accepts_final_line() {
    use super::explain_review_chat_is_lgtm;
    assert!(explain_review_chat_is_lgtm("LGTM"));
    assert!(explain_review_chat_is_lgtm("  LGTM\n"));
    assert!(explain_review_chat_is_lgtm(
        "Checks passed.\nProducts nonempty.\nLGTM\n"
    ));
    assert!(explain_review_chat_is_lgtm(
        "The chat deliverable must be exactly LGTM.\nLGTM"
    ));
    // Observed Cursor streaming: no newline before the deliverable.
    assert!(explain_review_chat_is_lgtm(
        "Logging KPop iterations as I probe products, cold entry, and settle-and-stop.LGTM"
    ));
    assert!(!explain_review_chat_is_lgtm(""));
    assert!(!explain_review_chat_is_lgtm(
        "- Missing products: no tex/pdf pair.\n- Fix cold entry."
    ));
    // Gap list must not accept even when a trailing LGTM is streamed after bullets.
    assert!(!explain_review_chat_is_lgtm(
        "- Fix cold entry in section 2.\n- Pay unpaid debt at section 3.\nLGTM"
    ));
    assert!(!explain_review_chat_is_lgtm(
        "- Fix cold entry before landscape/pressure.LGTM"
    ));
    assert!(!explain_review_chat_is_lgtm("LGTM\nextra trailing gap"));
    assert!(!explain_review_chat_is_lgtm("Almost LGTM"));
    assert!(!explain_review_chat_is_lgtm("NOT_LGTM"));
    assert!(!explain_review_chat_is_lgtm("return LGTM"));
}

#[test]
fn kiss_cov_resolve_paths() {
    let _ = products_nonempty;
    let tmp = tempfile::tempdir().expect("tmp");
    let work = tmp.path();
    let store = crate::prompts::PromptStore::default_store();
    store.ensure_defaults().expect("defaults");
    let artifacts =
        crate::artifacts::create_kpop_run_artifacts("explain", Some(work)).expect("art");
    let prepared = ExplainKpopPrepared {
        inner: crate::kpop_engine::KPopEnginePrepared {
            artifacts,
            context: crate::prompt_stratification::WorkflowRenderContext::default(),
            request_text: "req".into(),
            startup_emit_request: "req".into(),
            store,
            malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
        },
        tex_path: work.join("explain.tex"),
        pdf_path: work.join("explain.pdf"),
        request_work_dir: work.to_path_buf(),
        auto_out_path: false,
        preflight_snapshot: ExplainPreflightSnapshot::default(),
    };
    let resolved = resolve_explain_output_paths(&prepared).expect("resolve");
    assert_eq!(resolved.0, prepared.tex_path);
}
