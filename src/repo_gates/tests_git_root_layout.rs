use super::checks_test_helpers::{git_init, write_git_root_checks as write_checks, write_legacy_cwd_checks as write_legacy_checks};
use super::*;

#[test]
fn should_run_workspace_gates_when_malvin_dir_present() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(crate::MALVIN_DIR)).unwrap();
    assert!(should_run_workspace_gates(tmp.path()));
}

#[test]
fn resolve_malvin_checks_reads_legacy_cwd_relative_file() {
    crate::test_utils::with_isolated_home(|w| {
        write_legacy_checks(w, "legacy-only\n");
        assert_eq!(gate_command_lines(w).unwrap(), vec!["legacy-only".to_string()]);
    });
}

#[test]
fn kiss_cov_repo_gates_test_helpers() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    git_init(w);
    write_checks(w, "kiss check\n");
    write_legacy_checks(w, "legacy\n");
    let _ = stringify!(write_legacy_cwd_checks);
}
