use std::path::{Path, PathBuf};

use super::explain_preflight;

fn explain_preflight_in_cwd(
    cwd: &Path,
    request: &str,
) -> Result<(String, PathBuf), String> {
    let old = std::env::current_dir().expect("cwd");
    std::env::set_current_dir(cwd).expect("chdir");
    let result = explain_preflight(request);
    std::env::set_current_dir(old).expect("restore");
    result
}

#[test]
fn explain_proceeds_when_outputs_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (text, work_dir) = explain_preflight_in_cwd(tmp.path(), "topic").expect("ok");
    assert_eq!(text, "topic");
    assert_eq!(work_dir, PathBuf::from("."));
}

#[test]
fn explain_fails_when_tex_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("explain.tex"), "STALE\n").expect("write");
    let err = explain_preflight_in_cwd(tmp.path(), "topic").expect_err("exists");
    assert!(err.contains("refusing to overwrite"));
}

#[test]
fn explain_fails_when_pdf_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("explain.pdf"), b"%PDF").expect("write");
    let err = explain_preflight_in_cwd(tmp.path(), "topic").expect_err("exists");
    assert!(err.contains("refusing to overwrite"));
}

#[test]
fn explain_fails_when_nested_work_dir_output_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let notes = tmp.path().join("notes");
    std::fs::create_dir_all(&notes).expect("mkdir");
    std::fs::write(notes.join("topic.md"), "Explain this\n").expect("write");
    std::fs::write(notes.join("explain.tex"), "STALE\n").expect("write");
    let err = explain_preflight_in_cwd(tmp.path(), "notes/topic.md").expect_err("exists");
    assert!(err.contains("refusing to overwrite"));
}
