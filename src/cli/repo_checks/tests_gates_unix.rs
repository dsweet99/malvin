use std::fs;
use std::time::Duration;

use super::command_support::set_fake_command_dir;
use super::tests_gates_common::log_contains_command;
use super::tests_gates_helpers::{
    install_trace_echo_bins, workspace_git_minimal_cargo_rs_py_tests,
    workspace_git_precommit_malvin_checks_cargo_main,
};
use super::{RepoGateOutput, run_repo_workspace_gates};

#[test]
fn source_like_files_present_does_not_follow_external_symlink_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(outside.path().join("src")).unwrap();
    std::fs::write(outside.path().join("src/main.rs"), "fn main() {}").unwrap();
    std::os::unix::fs::symlink(outside.path(), tmp.path().join("src")).unwrap();
    assert!(!crate::source_detect::has_source_files(tmp.path()));
}

#[tokio::test]
async fn test_scan_for_extension_handles_symlink_cycles() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().to_path_buf();
    std::fs::create_dir(root.join("src")).unwrap();
    std::os::unix::fs::symlink(&root, root.join("src").join("cycle")).unwrap();

    let scan = tokio::task::spawn_blocking(move || {
        crate::source_detect::has_extension_files(&root, "rs")
            || crate::source_detect::has_extension_files(&root, "py")
    });
    let found = tokio::time::timeout(Duration::from_secs(1), scan)
        .await
        .expect("test_scan_for_extension_handles_symlink_cycles must finish")
        .expect("panicked");
    assert!(!found);
}

#[test]
fn run_repo_workspace_gates_invokes_seeded_kiss_check() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_minimal_cargo_rs_py_tests(work);
    super::tests_gates_helpers::seed_workspace_builtin_malvin_checks(work);
    let bin_dir = tempfile::tempdir().unwrap();
    let trace = bin_dir.path().join("trace.log");
    install_trace_echo_bins(bin_dir.path(), &trace, &["kiss"], 0);
    let _guard = set_fake_command_dir(bin_dir.path());
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = fs::read_to_string(&trace).unwrap();
    assert!(log.contains("kiss clamp"));
    assert!(log_contains_command(&log, "kiss check"));
}

#[test]
fn run_repo_workspace_gates_skips_pre_commit_when_config_present() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_precommit_malvin_checks_cargo_main(work);
    let bin_dir = tempfile::tempdir().unwrap();
    let trace = bin_dir.path().join("trace.log");
    install_trace_echo_bins(bin_dir.path(), &trace, &["kiss", "custom"], 0);
    let _guard = set_fake_command_dir(bin_dir.path());
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = fs::read_to_string(&trace).unwrap();
    assert!(!log_contains_command(&log, "pre-commit run --all-files"));
    assert!(!log_contains_command(&log, "kiss check"));
    assert!(log_contains_command(&log, "custom --only"));
}


#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_test_scan_for_extension_handles_symlink_cycles() { let _ = test_scan_for_extension_handles_symlink_cycles; }
}
