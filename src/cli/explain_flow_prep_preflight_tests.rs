use super::{
    discover_explain_outputs_in_work_dir, explain_preflight, ExplainPreflightSnapshot,
    EXPLAIN_PDF_BASENAME, EXPLAIN_TEX_BASENAME,
};
use crate::test_utils::with_cwd;

#[test]
fn explain_proceeds_when_outputs_missing() {
    let tmp = tempfile::tempdir().expect("tempdir");
    with_cwd(tmp.path(), || {
        let (text, work_dir, outputs, _snapshot) =
            explain_preflight("topic", EXPLAIN_TEX_BASENAME, false).expect("ok");
        let cwd = std::env::current_dir().expect("cwd");
        assert_eq!(text, "topic");
        assert_eq!(work_dir, std::path::PathBuf::from("."));
        assert_eq!(outputs.tex_path, cwd.join(EXPLAIN_TEX_BASENAME));
        assert_eq!(outputs.pdf_path, cwd.join(EXPLAIN_PDF_BASENAME));
    });
}

#[test]
fn explain_auto_mode_does_not_allocate_siblings_when_default_outputs_exist() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("explain.tex"), "STALE\n").expect("write");
    std::fs::write(tmp.path().join("explain.pdf"), b"%PDF").expect("write");
    with_cwd(tmp.path(), || {
        let (_text, _work_dir, outputs, snapshot) =
            explain_preflight("topic", EXPLAIN_TEX_BASENAME, false).expect("auto preflight");
        let cwd = std::env::current_dir().expect("cwd");
        assert_eq!(outputs.tex_path, cwd.join(EXPLAIN_TEX_BASENAME));
        assert!(!snapshot.pre_existing_tex_pdf.is_empty());
        assert!(!cwd.join("explain_1.tex").exists());
    });
}

#[test]
fn explain_explicit_default_allocates_explain_1_when_outputs_exist() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("explain.tex"), "STALE\n").expect("write");
    std::fs::write(tmp.path().join("explain.pdf"), b"%PDF").expect("write");
    with_cwd(tmp.path(), || {
        let (_text, _work_dir, outputs, _snapshot) =
            explain_preflight("topic", EXPLAIN_TEX_BASENAME, true).expect("allocates sibling");
        let cwd = std::env::current_dir().expect("cwd");
        assert_eq!(outputs.tex_path, cwd.join("explain_1.tex"));
        assert_eq!(outputs.pdf_path, cwd.join("explain_1.pdf"));
    });
}

#[test]
fn explain_explicit_default_allocates_explain_1_when_only_default_pdf_exists_without_tex() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("explain.pdf"), b"%PDF").expect("write");
    with_cwd(tmp.path(), || {
        let (_text, _work_dir, outputs, _snapshot) =
            explain_preflight("topic", EXPLAIN_TEX_BASENAME, true).expect("allocates sibling");
        let cwd = std::env::current_dir().expect("cwd");
        assert_eq!(outputs.tex_path, cwd.join("explain_1.tex"));
        assert_eq!(outputs.pdf_path, cwd.join("explain_1.pdf"));
    });
}

#[test]
fn explain_explicit_default_allocates_sibling_when_nested_work_dir_default_output_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let notes = tmp.path().join("notes");
    std::fs::create_dir_all(&notes).expect("mkdir");
    std::fs::write(notes.join("topic.md"), "Explain this\n").expect("write");
    std::fs::write(notes.join("explain.tex"), "STALE\n").expect("write");
    with_cwd(tmp.path(), || {
        let (_text, _work_dir, outputs, _snapshot) =
            explain_preflight("notes/topic.md", EXPLAIN_TEX_BASENAME, true).expect("allocates sibling");
        let cwd = std::env::current_dir().expect("cwd");
        assert_eq!(outputs.tex_path, cwd.join("notes/explain_1.tex"));
        assert_eq!(outputs.pdf_path, cwd.join("notes/explain_1.pdf"));
    });
}

#[test]
fn explain_fails_when_custom_out_path_exists() {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("docs")).expect("mkdir");
    std::fs::write(tmp.path().join("docs/paper.tex"), "STALE\n").expect("write");
    let err = with_cwd(tmp.path(), || explain_preflight("topic", "docs/paper.tex", true))
        .expect_err("exists");
    assert!(err.contains("refusing to overwrite"));
}

#[test]
fn explain_custom_out_path_resolves_against_cwd() {
    let tmp = tempfile::tempdir().expect("tempdir");
    with_cwd(tmp.path(), || {
        let (text, _work_dir, outputs, _snapshot) =
            explain_preflight("topic", "docs/paper.tex", true).expect("ok");
        let cwd = std::env::current_dir().expect("cwd");
        assert_eq!(text, "topic");
        assert_eq!(outputs.tex_path, cwd.join("docs/paper.tex"));
        assert_eq!(outputs.pdf_path, cwd.join("docs/paper.pdf"));
    });
}

fn seed_stale_explain_pair(dir: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let stale_tex = dir.join("explain.tex");
    let stale_pdf = dir.join("explain.pdf");
    std::fs::write(&stale_tex, "STALE\n").expect("write stale tex");
    std::fs::write(&stale_pdf, b"%PDF stale").expect("write stale pdf");
    (stale_tex, stale_pdf)
}

fn seed_gate_loop_exit_outputs(dir: &std::path::Path) {
    std::fs::write(dir.join("gate_loop_exit.tex"), "\\documentclass{article}").expect("write");
    std::fs::write(dir.join("gate_loop_exit.pdf"), b"%PDF").expect("write");
}

#[test]
fn discover_explain_outputs_finds_newest_non_preexisting_pair() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let (stale_tex, stale_pdf) = seed_stale_explain_pair(tmp.path());
    let snapshot = ExplainPreflightSnapshot {
        pre_existing_tex_pdf: std::iter::once(stale_tex)
            .chain(std::iter::once(stale_pdf))
            .collect(),
    };
    std::thread::sleep(std::time::Duration::from_millis(10));
    seed_gate_loop_exit_outputs(tmp.path());
    with_cwd(tmp.path(), || {
        let outputs =
            discover_explain_outputs_in_work_dir(std::path::Path::new("."), &snapshot).expect("discover");
        assert_eq!(outputs.tex_path.file_name().unwrap(), "gate_loop_exit.tex");
    });
}
