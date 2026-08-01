use super::*;
use super::checks_test_helpers::{git_init, write_git_root_checks as write_checks};
use std::fs;

#[test]
fn load_malvin_checks_skips_comments_and_blank_lines() {
    let tmp = tempfile::tempdir().unwrap();
    let checks_path = tmp.path().join(".malvin/checks");
    std::fs::create_dir_all(checks_path.parent().unwrap()).unwrap();
    std::fs::write(&checks_path, "# header\n\ncustom-a\n# tail\n").unwrap();
    let lines = load_malvin_checks(&checks_path).unwrap();
    assert_eq!(lines, vec!["custom-a".to_string()]);
}

#[test]
fn prompt_quality_gates_markdown_matches_checks_file_verbatim() {
    crate::test_utils::with_isolated_home(|w| {
        std::fs::create_dir_all(w.join(".malvin")).unwrap();
        std::fs::write(
            w.join(".malvin/checks"),
            "# lint gates\ncustom-a\ncargo nextest run\n",
        )
        .unwrap();
        let md = prompt_quality_gates_markdown(w).unwrap();
        assert!(md.contains("`custom-a`"));
        assert!(md.contains("`cargo nextest run`"));
        assert!(!md.contains("# lint"));
        assert!(!md.contains("--partition"));
    });
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
fn should_run_workspace_gates_when_git_present() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir(tmp.path().join(".git")).unwrap();
    assert!(should_run_workspace_gates(tmp.path()));
}

#[test]
fn format_quality_gates_markdown_lists_commands() {
    let lines = vec!["make lint".to_string(), "ruff check .".to_string()];
    let md = format_quality_gates_markdown(&lines);
    assert!(md.contains("`make lint`"));
    assert!(md.contains("`ruff check .`"));
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
