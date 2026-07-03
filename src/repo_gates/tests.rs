use super::*;
use super::checks_test_helpers::{git_init, write_git_root_checks as write_checks};
use std::fs;

#[test]
fn builtin_gate_command_lines_returns_kiss_only_for_rust_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::create_dir(w.join(".git")).unwrap();
    fs::write(
        w.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .unwrap();
    let g = builtin_gate_command_lines(w);
    assert_eq!(g, vec![KISS_CHECK_COMMAND.to_string()]);
}

#[test]
fn builtin_gate_command_lines_returns_kiss_only_for_python_repo() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::create_dir(w.join(".git")).unwrap();
    fs::write(w.join("main.py"), "x=1\n").unwrap();
    fs::write(w.join("test_foo.py"), "def test_x():\n    assert True\n").unwrap();
    let g = builtin_gate_command_lines(w);
    assert_eq!(g, vec![KISS_CHECK_COMMAND.to_string()]);
}

#[test]
fn gate_command_lines_uses_only_malvin_checks_when_present() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    git_init(w);
    fs::write(
        w.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .unwrap();
    write_checks(w, "custom-a\ncustom-b\n");
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
fn ensure_default_malvin_checks_file_writes_kiss_only() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    git_init(w);
    fs::write(
        w.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .unwrap();
    let checks_path = crate::malvin_checks_path(w);
    assert!(!checks_path.exists());
    let expected = vec![KISS_CHECK_COMMAND.to_string()];
    ensure_default_malvin_checks_file(w).unwrap();
    assert!(checks_path.is_file());
    assert_eq!(load_malvin_checks(&checks_path).unwrap(), expected);
    ensure_default_malvin_checks_file(w).unwrap();
    assert_eq!(load_malvin_checks(&checks_path).unwrap(), expected);
}

#[test]
fn prompt_quality_gates_includes_kiss_only_for_seeded_checks() {
    crate::test_utils::with_isolated_home(|w| {
        fs::write(
            w.join("Cargo.toml"),
            "[package]\nname='x'\nversion='0.1.0'\n",
        )
        .unwrap();
        ensure_default_malvin_checks_file(w).unwrap();
        let md = prompt_quality_gates_markdown(w).unwrap();
        assert!(md.contains(&format!("- `{KISS_CHECK_COMMAND}`")));
        assert_eq!(md.matches('`').count(), 2);
    });
}

#[test]
fn should_run_workspace_gates_when_git_present() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join(".git")).unwrap();
    assert!(should_run_workspace_gates(tmp.path()));
}

#[test]
fn format_quality_gates_markdown_lists_commands() {
    let lines = vec!["kiss check".to_string(), "lint check".to_string()];
    let md = format_quality_gates_markdown(&lines);
    assert!(md.contains("`kiss check`"));
    assert!(md.contains("`lint check`"));
}

#[test]
fn prompt_quality_gates_markdown_ephemeral_errors_when_checks_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    git_init(w);
    let checks_path = crate::malvin_checks_path(w);
    assert!(!checks_path.exists());
    let err = prompt_quality_gates_markdown_ephemeral(w).unwrap_err();
    assert!(err.contains("is missing"), "unexpected error: {err}");
    assert!(!checks_path.exists());
}

#[test]
fn prompt_quality_gates_markdown_errors_when_malvin_checks_missing() {
    crate::test_utils::with_isolated_home(|w| {
        let err = prompt_quality_gates_markdown(w).unwrap_err();
        assert!(
            err.contains("is missing"),
            "unexpected error message: {err}"
        );
    });
}
