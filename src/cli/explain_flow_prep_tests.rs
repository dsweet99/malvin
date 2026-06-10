use super::{
    explain_kpop_request, explain_output_paths, explain_preflight, prepare_explain_kpop_prompt_store,
    EXPLAIN_PDF_BASENAME, EXPLAIN_TEX_BASENAME,
};
use crate::artifacts::create_kpop_run_artifacts;
use crate::cli::WorkflowCliOptions;

#[test]
fn explain_output_paths_use_fixed_basenames() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (tex, pdf) = explain_output_paths(tmp.path());
    assert_eq!(tex.file_name().unwrap(), EXPLAIN_TEX_BASENAME);
    assert_eq!(pdf.file_name().unwrap(), EXPLAIN_PDF_BASENAME);
}

#[test]
fn explain_kpop_request_renders_request_and_output_paths() {
    let store = prepare_explain_kpop_prompt_store(WorkflowCliOptions { force: true }).expect("store");
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_kpop_run_artifacts("explain", Some(tmp.path())).expect("artifacts");
    let work_dir = tmp.path();
    let rendered = explain_kpop_request(&store, &artifacts, "gate loop exit", work_dir).expect("render");
    assert!(rendered.contains("gate loop exit"));
    assert!(rendered.contains("explain.tex"));
    assert!(rendered.contains("explain.pdf"));
    assert!(rendered.contains("Satisfy all constraints"));
    assert!(rendered.contains("Scope Constraints"));
}

#[test]
fn explain_preflight_literal_request_uses_dot_work_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(tmp.path()).expect("chdir");
    let result = explain_preflight("  topic  ");
    std::env::set_current_dir(old).expect("restore");
    let (text, work_dir) = result.expect("ok");
    assert_eq!(text, "topic");
    assert_eq!(work_dir, std::path::PathBuf::from("."));
}

#[test]
fn explain_preflight_md_file_uses_parent_work_dir() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    std::env::set_current_dir(root).unwrap();
    let md_path = root.join("notes/topic.md");
    std::fs::create_dir_all(md_path.parent().unwrap()).unwrap();
    std::fs::write(&md_path, "Explain this\n").unwrap();
    let (text, work_dir) = explain_preflight("notes/topic.md").unwrap();
    assert_eq!(text.trim(), "Explain this");
    assert_eq!(work_dir, crate::artifacts::work_dir_for_path(&md_path));
}
