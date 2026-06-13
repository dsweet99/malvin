use super::{explain_preflight, EXPLAIN_PDF_BASENAME, EXPLAIN_TEX_BASENAME};
use crate::test_utils::with_cwd;

#[test]
fn explain_proceeds_when_outputs_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    with_cwd(tmp.path(), || {
        let (text, outputs) = explain_preflight("topic", EXPLAIN_TEX_BASENAME).expect("ok");
        let cwd = std::env::current_dir().expect("cwd");
        assert_eq!(text, "topic");
        assert_eq!(outputs.tex_path, cwd.join(EXPLAIN_TEX_BASENAME));
        assert_eq!(outputs.pdf_path, cwd.join(EXPLAIN_PDF_BASENAME));
    });
}

#[test]
fn explain_allocates_explain_1_when_default_outputs_exist() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("explain.tex"), "STALE\n").expect("write");
    std::fs::write(tmp.path().join("explain.pdf"), b"%PDF").expect("write");
    with_cwd(tmp.path(), || {
        let (_text, outputs) =
            explain_preflight("topic", EXPLAIN_TEX_BASENAME).expect("allocates sibling");
        assert_eq!(outputs.tex_path, tmp.path().join("explain_1.tex"));
        assert_eq!(outputs.pdf_path, tmp.path().join("explain_1.pdf"));
    });
}

#[test]
fn explain_allocates_explain_1_when_only_default_pdf_exists_without_tex() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("explain.pdf"), b"%PDF").expect("write");
    with_cwd(tmp.path(), || {
        let (_text, outputs) =
            explain_preflight("topic", EXPLAIN_TEX_BASENAME).expect("allocates sibling");
        assert_eq!(outputs.tex_path, tmp.path().join("explain_1.tex"));
        assert_eq!(outputs.pdf_path, tmp.path().join("explain_1.pdf"));
    });
}

#[test]
fn explain_allocates_sibling_when_nested_work_dir_default_output_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let notes = tmp.path().join("notes");
    std::fs::create_dir_all(&notes).expect("mkdir");
    std::fs::write(notes.join("topic.md"), "Explain this\n").expect("write");
    std::fs::write(notes.join("explain.tex"), "STALE\n").expect("write");
    with_cwd(tmp.path(), || {
        let (_text, outputs) =
            explain_preflight("notes/topic.md", EXPLAIN_TEX_BASENAME).expect("allocates sibling");
        assert_eq!(outputs.tex_path, notes.join("explain_1.tex"));
        assert_eq!(outputs.pdf_path, notes.join("explain_1.pdf"));
    });
}

#[test]
fn explain_fails_when_custom_out_path_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("docs")).expect("mkdir");
    std::fs::write(tmp.path().join("docs/paper.tex"), "STALE\n").expect("write");
    let err = with_cwd(tmp.path(), || explain_preflight("topic", "docs/paper.tex")).expect_err("exists");
    assert!(err.contains("refusing to overwrite"));
}

#[test]
fn explain_custom_out_path_resolves_against_cwd() {
    let tmp = tempfile::tempdir().expect("tempdir");
    with_cwd(tmp.path(), || {
        let (text, outputs) = explain_preflight("topic", "docs/paper.tex").expect("ok");
        let cwd = std::env::current_dir().expect("cwd");
        assert_eq!(text, "topic");
        assert_eq!(outputs.tex_path, cwd.join("docs/paper.tex"));
        assert_eq!(outputs.pdf_path, cwd.join("docs/paper.pdf"));
    });
}
