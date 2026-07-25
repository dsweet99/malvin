use super::*;
use super::super::prep::ExplainPreflightSnapshot;
use super::super::run_startup::ExplainKpopPrepared;

#[test]
fn validate_products() {
    let _ = validate_explain_output;
    let _ = products_nonempty;
    let tmp = tempfile::tempdir().expect("tmp");
    let tex = tmp.path().join("explain.tex");
    let pdf = tmp.path().join("explain.pdf");
    std::fs::write(&tex, "x").unwrap();
    std::fs::write(&pdf, "y").unwrap();
    validate_explain_output(&tex, &pdf).expect("ok");
    assert!(products_nonempty(&tex, &pdf));
}

#[test]
fn resolve_paths_witness() {
    let _ = resolve_explain_output_paths;
}

#[test]
fn resolve_paths_explicit() {
    let tmp = tempfile::tempdir().expect("tmp");
    let work = tmp.path();
    let tex = work.join("a.tex");
    let pdf = work.join("a.pdf");
    let store = crate::prompts::PromptStore::default_store();
    let _ = store.ensure_defaults();
    let artifacts = crate::artifacts::create_kpop_run_artifacts("explain", Some(work)).expect("a");
    let prepared = ExplainKpopPrepared {
        inner: crate::kpop_engine::KPopEnginePrepared {
            artifacts,
            context: crate::prompt_stratification::WorkflowRenderContext::default(),
            request_text: "r".into(),
            startup_emit_request: "r".into(),
            store,
            malvin_checks_backup: crate::artifacts::MalvinChecksBackup::Missing,
        },
        tex_path: tex.clone(),
        pdf_path: pdf.clone(),
        request_work_dir: work.to_path_buf(),
        auto_out_path: false,
        preflight_snapshot: ExplainPreflightSnapshot::default(),
    };
    let (t, p) = resolve_explain_output_paths(&prepared).expect("paths");
    assert_eq!((t, p), (tex, pdf));
}
