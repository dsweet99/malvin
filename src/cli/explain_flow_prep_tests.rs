use super::{
    explain_kpop_request, explain_output_paths, explain_pdf_path_from_tex, explain_preflight,
    explain_resolved_output_paths, explain_revise_doc_path, prepare_explain_kpop_prompt_store,
    EXPLAIN_PDF_BASENAME, EXPLAIN_TEX_BASENAME,
};
use crate::artifacts::create_kpop_run_artifacts;
use crate::cli::WorkflowCliOptions;
use crate::test_utils::with_cwd;

#[test]
fn explain_output_paths_resolve_dot_work_dir_in_cwd() {
    let tmp = tempfile::tempdir().expect("tempdir");
    with_cwd(tmp.path(), || {
        let cwd = std::env::current_dir().expect("cwd");
        let outputs = explain_output_paths(std::path::Path::new("."));
        assert_eq!(outputs.tex_path, cwd.join(EXPLAIN_TEX_BASENAME));
        assert_eq!(outputs.pdf_path, cwd.join(EXPLAIN_PDF_BASENAME));
    });
}

#[test]
fn explain_output_paths_use_fixed_basenames() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let outputs = explain_output_paths(tmp.path());
    assert_eq!(outputs.tex_path.file_name().unwrap(), EXPLAIN_TEX_BASENAME);
    assert_eq!(outputs.pdf_path.file_name().unwrap(), EXPLAIN_PDF_BASENAME);
}

#[test]
fn explain_resolved_output_paths_keep_default_in_request_work_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    with_cwd(tmp.path(), || {
        let cwd = std::env::current_dir().expect("cwd");
        let notes = cwd.join("notes");
        std::fs::create_dir_all(&notes).expect("mkdir");
        let outputs =
            explain_resolved_output_paths(std::path::Path::new("notes"), EXPLAIN_TEX_BASENAME)
                .expect("resolve");
        assert_eq!(outputs.tex_path, notes.join(EXPLAIN_TEX_BASENAME));
        assert_eq!(outputs.pdf_path, notes.join(EXPLAIN_PDF_BASENAME));
    });
}

#[test]
fn explain_resolved_output_paths_use_custom_out_path_in_cwd() {
    let tmp = tempfile::tempdir().expect("tempdir");
    with_cwd(tmp.path(), || {
        let cwd = std::env::current_dir().expect("cwd");
        let outputs =
            explain_resolved_output_paths(std::path::Path::new("."), "docs/paper.tex").expect("resolve");
        assert_eq!(outputs.tex_path, cwd.join("docs/paper.tex"));
        assert_eq!(outputs.pdf_path, cwd.join("docs/paper.pdf"));
    });
}

#[test]
fn explain_pdf_path_from_tex_replaces_extension() {
    let tex = std::path::PathBuf::from("out/explain.tex");
    assert_eq!(
        explain_pdf_path_from_tex(&tex),
        std::path::PathBuf::from("out/explain.pdf")
    );
}

#[test]
fn explain_kpop_request_renders_request_and_output_paths() {
    let store = prepare_explain_kpop_prompt_store(WorkflowCliOptions { force: true }).expect("store");
    let tmp = tempfile::tempdir().expect("tempdir");
    let artifacts = create_kpop_run_artifacts("explain", Some(tmp.path())).expect("artifacts");
    let outputs = explain_resolved_output_paths(tmp.path(), "docs/paper.tex").expect("resolve");
    let rendered =
        explain_kpop_request(&store, &artifacts, "gate loop exit", &outputs).expect("render");
    assert!(rendered.contains("gate loop exit"));
    assert!(rendered.contains("docs/paper.tex"));
    assert!(rendered.contains("docs/paper.pdf"));
    assert!(rendered.contains("Satisfy all constraints"));
    assert!(rendered.contains("Scope Constraints"));
}

#[test]
fn explain_preflight_literal_request_uses_dot_work_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    with_cwd(tmp.path(), || {
        let cwd = std::env::current_dir().expect("cwd");
        let (text, outputs) = explain_preflight("  topic  ", EXPLAIN_TEX_BASENAME).expect("ok");
        assert_eq!(text, "topic");
        assert_eq!(outputs.tex_path, cwd.join(EXPLAIN_TEX_BASENAME));
        assert_eq!(outputs.pdf_path, cwd.join(EXPLAIN_PDF_BASENAME));
    });
}

#[test]
fn explain_preflight_md_file_uses_parent_work_dir() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    with_cwd(tmp.path(), || {
        let cwd = std::env::current_dir().expect("cwd");
        let md_path = cwd.join("notes/topic.md");
        std::fs::create_dir_all(md_path.parent().unwrap()).unwrap();
        std::fs::write(&md_path, "Explain this\n").unwrap();
        let (text, outputs) = explain_preflight("notes/topic.md", EXPLAIN_TEX_BASENAME).unwrap();
        assert_eq!(text.trim(), "Explain this");
        assert_eq!(outputs.tex_path, cwd.join("notes/explain.tex"));
        assert_eq!(outputs.pdf_path, cwd.join("notes/explain.pdf"));
    });
}

#[test]
fn explain_revise_doc_path_uses_resolved_tex_in_request_work_dir() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = tempfile::tempdir().unwrap();
    with_cwd(tmp.path(), || {
        let cwd = std::env::current_dir().expect("cwd");
        let md_path = cwd.join("notes/topic.md");
        std::fs::create_dir_all(md_path.parent().unwrap()).unwrap();
        std::fs::write(&md_path, "Explain this\n").unwrap();
        let doc_path = explain_revise_doc_path("notes/topic.md", EXPLAIN_TEX_BASENAME).unwrap();
        assert_eq!(doc_path, "notes/explain.tex");
    });
}

#[test]
fn explain_revise_doc_path_uses_custom_out_path_in_cwd() {
    let tmp = tempfile::tempdir().expect("tempdir");
    with_cwd(tmp.path(), || {
        let doc_path = explain_revise_doc_path("topic", "docs/paper.tex").expect("resolve");
        assert_eq!(doc_path, "docs/paper.tex");
    });
}
