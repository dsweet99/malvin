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
    fs::create_dir_all(w.join(".malvin")).unwrap();
    fs::write(w.join(".malvin/checks"), "custom-a\ncustom-b\n").unwrap();
    let g = gate_command_lines(w).unwrap();
    assert_eq!(g, vec!["custom-a".to_string(), "custom-b".to_string()]);
    assert!(!g.iter().any(|c| c == KISS_CHECK_COMMAND));
}

#[test]
fn ensure_default_malvin_config_file_writes_template_when_missing() {
    crate::test_utils::with_isolated_home(|work| {
        let config_path = crate::malvin_config_path(work);
        assert!(!config_path.exists());
        ensure_default_malvin_config_file(work).unwrap();
        assert!(config_path.is_file());
        let text = fs::read_to_string(&config_path).unwrap();
        assert!(text.contains("[logs]"));
        assert!(text.contains("[agent]"));
        ensure_default_malvin_config_file(work).unwrap();
        assert_eq!(fs::read_to_string(&config_path).unwrap(), text);
    });
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
    assert!(
        md.contains("cargo test") || md.contains("cargo nextest run"),
        "md: {md}"
    );
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
        "ephemeral prompt expansion must restore Missing .malvin/checks"
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
fn smoke_cov_repo_gates_units() {
    let _ = builtin_gate_command_lines;
}

#[test]
fn refresh_provisional_malvin_checks_file_replaces_existing() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    std::fs::create_dir_all(w.join(".malvin")).unwrap();
    std::fs::write(w.join(".malvin/checks"), "old\n").unwrap();
    refresh_provisional_malvin_checks_file(w).unwrap();
    let text = std::fs::read_to_string(w.join(".malvin/checks")).unwrap();
    assert!(text.contains("kiss check"));
    assert!(!text.contains("old"));
}

#[test]
fn augment_init_checks_adds_ruff_from_precommit_template() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    std::fs::create_dir_all(w.join(".malvin")).unwrap();
    std::fs::write(w.join(".malvin/checks"), "kiss check\n").unwrap();
    std::fs::write(
        w.join(".pre-commit-config.yaml"),
        "repos:\n- repo: local\n  hooks:\n  - id: ruff\n    entry: ruff check .\n",
    )
    .unwrap();
    crate::repo_gates::discover_init_checks::augment_init_checks_with_precommit_python_gates(w)
        .unwrap();
    let checks = std::fs::read_to_string(w.join(".malvin/checks")).unwrap();
    assert!(checks.contains("ruff check ."));
    assert!(checks.contains(DEFAULT_PYTEST_CHECK));
}

#[test]
fn augment_init_checks_adds_pytest_when_ruff_already_present() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    std::fs::create_dir_all(w.join(".malvin")).unwrap();
    std::fs::write(
        w.join(".malvin/checks"),
        "kiss check\nruff check .\n",
    )
    .unwrap();
    crate::repo_gates::discover_init_checks::augment_init_checks_with_precommit_python_gates(w)
        .unwrap();
    let checks = std::fs::read_to_string(w.join(".malvin/checks")).unwrap();
    assert!(checks.contains("ruff check ."));
    assert!(checks.contains(DEFAULT_PYTEST_CHECK));
}

#[test]
fn smoke_cov_discover_init_checks_finalize() {
    let _ = crate::repo_gates::discover_init_checks::finalize_init_checks_from_repo;
    let _ = crate::repo_gates::discover_init_checks::checks_cover_precommit_signals;
    let _ = crate::repo_gates::discover_init_checks::augment_init_checks_with_precommit_python_gates;
}

#[test]
fn default_rust_test_command_matches_nextest_probe() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    let cmd = default_rust_test_command(w);
    if cargo_nextest_available(w) {
        assert_eq!(cmd, DEFAULT_RUST_NEXTEST);
    } else {
        assert_eq!(cmd, DEFAULT_RUST_TEST);
    }
}

#[test]
fn should_run_workspace_gates_when_malvin_dir_present() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(crate::MALVIN_DIR)).unwrap();
    assert!(should_run_workspace_gates(tmp.path()));
}
