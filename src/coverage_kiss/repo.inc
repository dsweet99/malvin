// Quality gate discovery smoke tests.

fn tmp_repo_with_git_and_py() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join(".git")).expect("fake git");
    let py_dir = tmp.path().join("py");
    std::fs::create_dir_all(&py_dir).expect("py dir");
    std::fs::write(py_dir.join("mod.py"), "x=1\n").expect("py file");
    tmp
}

#[test]
fn smoke_repo_gates_builtin() {
    assert!(!crate::repo_gates::KISS_CHECK_COMMAND.is_empty());
    let tmp = tmp_repo_with_git_and_py();
    assert!(crate::repo_gates::should_run_workspace_gates(tmp.path()));

    let lines = crate::repo_gates::gate_command_lines(tmp.path()).expect("builtin gates");
    assert!(lines.iter().any(|l| l.contains("kiss check")));
    let lines_with_py = crate::repo_gates::gate_command_lines(tmp.path()).expect("gates with py");
    assert!(
        lines_with_py.iter().any(|l| l.contains("ruff")),
        "{lines_with_py:?}"
    );
}

#[test]
fn smoke_repo_gates_malvin_checks_roundtrip() {
    let tmp = tmp_repo_with_git_and_py();
    let lines = crate::repo_gates::gate_command_lines(tmp.path()).expect("builtin");
    crate::repo_gates::ensure_default_malvin_checks_file(tmp.path()).expect("default checks");
    let md = crate::repo_gates::prompt_quality_gates_markdown(tmp.path()).expect("qm");
    assert!(!md.is_empty());
    let fmt = crate::repo_gates::format_quality_gates_markdown(&lines);
    assert!(fmt.contains('`'));

    let loaded = crate::repo_gates::load_malvin_checks(&tmp.path().join(crate::repo_gates::MALVIN_CHECKS_FILE))
        .expect("load");
    assert!(!loaded.is_empty());
    crate::repo_gates::gate_command_lines_for_workspace_run(tmp.path()).expect("workspace lines");
}
