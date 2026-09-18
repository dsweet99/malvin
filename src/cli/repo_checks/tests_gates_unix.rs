use std::fs;

use super::command_support::set_fake_command_dir;
use super::tests_gates_common::log_contains_command;
use super::tests_gates_helpers::{
    install_trace_echo_bins, workspace_git_minimal_cargo_rs_py_tests,
    workspace_git_precommit_malvin_checks_cargo_main,
};
use super::{RepoGateOutput, run_repo_workspace_gates};

#[test]
fn run_repo_workspace_gates_invokes_seeded_checks() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_minimal_cargo_rs_py_tests(work);
    super::tests_gates_helpers::workspace_git_malvin_checks_line(work, "true\n");
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
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
    assert!(!log_contains_command(&log, "true"));
    assert!(log_contains_command(&log, "custom --only"));
}
