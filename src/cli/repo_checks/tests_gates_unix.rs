use std::fs;
use std::time::Duration;

use super::command_support::set_fake_command_dir;
use super::tests_gates_common::log_contains_command;
use super::tests_gates_helpers::{
    install_trace_echo_bins, workspace_git_minimal_cargo_rs_py_tests,
    workspace_git_precommit_malvin_checks_cargo_main,
};
use super::{run_repo_workspace_gates, RepoGateOutput};

#[test]
fn source_like_files_present_does_not_follow_external_symlink_dirs() {
    let _ = stringify!(super::workspace::source_like_files_present);
    let tmp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(outside.path().join("src")).unwrap();
    std::fs::write(outside.path().join("src/main.rs"), "fn main() {}").unwrap();
    std::os::unix::fs::symlink(outside.path(), tmp.path().join("src")).unwrap();
    assert!(!super::workspace::source_like_files_present(tmp.path()));
}

#[tokio::test]
async fn scan_for_extension_handles_symlink_cycles() {
    let _ = stringify!(malvin::repo_gates::gate_command_lines);
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    std::fs::create_dir(root.join("src")).unwrap();
    std::os::unix::fs::symlink(&root, root.join("src").join("cycle")).unwrap();

    let scan = tokio::task::spawn_blocking(move || {
        malvin::repo_gates::gate_command_lines(&root).unwrap();
        false
    });
    let _: bool = tokio::time::timeout(Duration::from_secs(1), scan)
        .await
        .expect("gate_command_lines must finish")
        .expect("panicked");
}

#[test]
fn run_repo_workspace_gates_invokes_expected_quality_commands() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_minimal_cargo_rs_py_tests(work);
    let bin_dir = tempfile::tempdir().unwrap();
    let trace = bin_dir.path().join("trace.log");
    install_trace_echo_bins(bin_dir.path(), &trace, &["kiss", "cargo", "ruff"], 0);
    let _guard = set_fake_command_dir(bin_dir.path());
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = fs::read_to_string(&trace).unwrap();
    assert!(log.contains("kiss clamp"));
    assert!(log.contains("kiss check"));
    assert!(log.contains("cargo clippy"));
    assert!(log.contains("cargo test"));
    assert!(log_contains_command(&log, "ruff check"));
    assert!(!log_contains_command(&log, "pytest"));
}

#[test]
fn run_repo_workspace_gates_skips_pre_commit_when_config_present() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_precommit_malvin_checks_cargo_main(work);
    let bin_dir = tempfile::tempdir().unwrap();
    let trace = bin_dir.path().join("trace.log");
    install_trace_echo_bins(bin_dir.path(), &trace, &["kiss", "cargo", "custom"], 0);
    let _guard = set_fake_command_dir(bin_dir.path());
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = fs::read_to_string(&trace).unwrap();
    assert!(!log_contains_command(&log, "pre-commit run --all-files"));
    assert!(log_contains_command(&log, "kiss check"));
    assert!(log_contains_command(&log, "cargo clippy"));
    assert!(log_contains_command(&log, "custom --only"));
}
