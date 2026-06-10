use std::path::Path;

use super::{explain_preflight, EXPLAIN_PDF_BASENAME, EXPLAIN_TEX_BASENAME};

fn explain_preflight_in_cwd(
    cwd: &Path,
    request: &str,
    out_path: &str,
) -> Result<(String, super::ExplainResolvedOutputs), String> {
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(cwd).expect("chdir");
    let result = explain_preflight(request, out_path);
    std::env::set_current_dir(old).expect("restore");
    result
}

#[test]
fn explain_proceeds_when_outputs_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (text, outputs) =
        explain_preflight_in_cwd(tmp.path(), "topic", EXPLAIN_TEX_BASENAME).expect("ok");
    assert_eq!(text, "topic");
    assert_eq!(outputs.tex_path, tmp.path().join(EXPLAIN_TEX_BASENAME));
    assert_eq!(outputs.pdf_path, tmp.path().join(EXPLAIN_PDF_BASENAME));
}

#[test]
fn explain_fails_when_tex_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("explain.tex"), "STALE\n").expect("write");
    let err = explain_preflight_in_cwd(tmp.path(), "topic", EXPLAIN_TEX_BASENAME).expect_err("exists");
    assert!(err.contains("refusing to overwrite"));
}

#[test]
fn explain_fails_when_pdf_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("explain.pdf"), b"%PDF").expect("write");
    let err = explain_preflight_in_cwd(tmp.path(), "topic", EXPLAIN_TEX_BASENAME).expect_err("exists");
    assert!(err.contains("refusing to overwrite"));
}

#[test]
fn explain_fails_when_nested_work_dir_output_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let notes = tmp.path().join("notes");
    std::fs::create_dir_all(&notes).expect("mkdir");
    std::fs::write(notes.join("topic.md"), "Explain this\n").expect("write");
    std::fs::write(notes.join("explain.tex"), "STALE\n").expect("write");
    let err = explain_preflight_in_cwd(tmp.path(), "notes/topic.md", EXPLAIN_TEX_BASENAME).expect_err("exists");
    assert!(err.contains("refusing to overwrite"));
}

#[test]
fn explain_fails_when_custom_out_path_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("docs")).expect("mkdir");
    std::fs::write(tmp.path().join("docs/paper.tex"), "STALE\n").expect("write");
    let err = explain_preflight_in_cwd(tmp.path(), "topic", "docs/paper.tex").expect_err("exists");
    assert!(err.contains("refusing to overwrite"));
}

#[test]
fn explain_custom_out_path_resolves_against_cwd() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (text, outputs) =
        explain_preflight_in_cwd(tmp.path(), "topic", "docs/paper.tex").expect("ok");
    assert_eq!(text, "topic");
    assert_eq!(outputs.tex_path, tmp.path().join("docs/paper.tex"));
    assert_eq!(outputs.pdf_path, tmp.path().join("docs/paper.pdf"));
}
