use super::*;
use std::fs;

#[test]
fn gate_command_lines_skips_ruff_when_no_python() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::create_dir(w.join(".git")).unwrap();
    fs::write(
        w.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .unwrap();
    let g = gate_command_lines(w).unwrap();
    assert!(g.iter().any(|c| c == KISS_CHECK_COMMAND));
    assert!(!g.iter().any(|c| c.starts_with("ruff")));
    assert!(g.iter().any(|c| c.starts_with("cargo clippy")));
}

#[test]
fn gate_command_lines_skips_pytest_without_test_named_py() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::create_dir(w.join(".git")).unwrap();
    fs::write(w.join("main.py"), "x=1\n").unwrap();
    let g = gate_command_lines(w).unwrap();
    assert!(g.iter().any(|c| c == "ruff check ."));
    assert!(!g.iter().any(|c| c.contains("pytest")));
}

#[test]
fn gate_command_lines_runs_pytest_when_test_module_present() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::create_dir(w.join(".git")).unwrap();
    fs::write(w.join("test_foo.py"), "def test_x():\n    assert True\n").unwrap();
    let g = gate_command_lines(w).unwrap();
    assert!(g.iter().any(|c| c == DEFAULT_PYTEST_CHECK));
}

#[test]
fn gate_command_lines_uses_only_malvin_checks_when_present() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::create_dir(w.join(".git")).unwrap();
    fs::write(
        w.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .unwrap();
    fs::write(w.join(".malvin_checks"), "custom-a\ncustom-b\n").unwrap();
    let g = gate_command_lines(w).unwrap();
    assert_eq!(g, vec!["custom-a".to_string(), "custom-b".to_string()]);
    assert!(!g.iter().any(|c| c == KISS_CHECK_COMMAND));
}

#[test]
fn ensure_default_malvin_checks_file_writes_builtin_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::create_dir(w.join(".git")).unwrap();
    fs::write(
        w.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .unwrap();
    let checks_path = w.join(MALVIN_CHECKS_FILE);
    assert!(!checks_path.exists());
    let expected = gate_command_lines(w).unwrap();
    ensure_default_malvin_checks_file(w).unwrap();
    assert!(checks_path.is_file());
    assert_eq!(load_malvin_checks(&checks_path).unwrap(), expected);
    ensure_default_malvin_checks_file(w).unwrap();
    assert_eq!(load_malvin_checks(&checks_path).unwrap(), expected);
}

#[test]
fn gate_command_lines_for_workspace_run_matches_file_after_ensure() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::create_dir(w.join(".git")).unwrap();
    fs::write(
        w.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .unwrap();
    let a = gate_command_lines_for_workspace_run(w).unwrap();
    let b = gate_command_lines(w).unwrap();
    assert_eq!(a, b);
}

#[test]
fn prompt_quality_gates_includes_rust_builtin_without_git_when_cargo_toml_present() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::write(
        w.join("Cargo.toml"),
        "[package]\nname='x'\nversion='0.1.0'\n",
    )
    .unwrap();
    ensure_default_malvin_checks_file(w).unwrap();
    let md = prompt_quality_gates_markdown(w).unwrap();
    assert!(md.contains(&format!("- `{KISS_CHECK_COMMAND}`")));
    assert!(md.contains("cargo clippy"));
    assert!(md.contains("cargo test"));
}

#[test]
fn should_run_workspace_gates_when_git_present() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join(".git")).unwrap();
    assert!(should_run_workspace_gates(tmp.path()));
}

#[test]
fn format_quality_gates_markdown_lists_commands() {
    let lines = vec!["kiss check".to_string(), "cargo test".to_string()];
    let md = format_quality_gates_markdown(&lines);
    assert!(md.contains("`kiss check`"));
    assert!(md.contains("`cargo test`"));
}

#[test]
fn prompt_quality_gates_markdown_ephemeral_restores_missing_malvin_checks() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::create_dir(w.join(".git")).unwrap();
    fs::write(
        w.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .unwrap();
    let checks_path = w.join(MALVIN_CHECKS_FILE);
    assert!(!checks_path.exists());
    let md = prompt_quality_gates_markdown_ephemeral(w).unwrap();
    assert!(md.contains(&format!("- `{KISS_CHECK_COMMAND}`")));
    assert!(
        !checks_path.exists(),
        "ephemeral prompt expansion must restore Missing .malvin_checks"
    );
}

#[test]
fn prompt_quality_gates_markdown_errors_when_malvin_checks_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    let err = prompt_quality_gates_markdown(w).unwrap_err();
    assert!(
        err.contains("is missing"),
        "unexpected error message: {err}"
    );
}

#[test]
fn kiss_stringify_repo_gates_units() {
    let _ = stringify!(builtin_gate_command_lines);
}
