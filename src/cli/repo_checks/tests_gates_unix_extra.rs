use std::fs;

use malvin::repo_gates;

use super::command_support::set_fake_command_dir;
use super::tests_gates_common::log_contains_command;
use super::tests_gates_helpers::{
    install_trace_echo_bins, workspace_git_cargo_main_only, workspace_git_kissconfig_90_cargo_rs_py,
    workspace_git_malvin_checks_line, workspace_git_minimal_cargo_rs_py_tests,
    write_executable_script, write_trace_echo_script,
};
use super::{prepare_repo_workspace, run_repo_workspace_gates, RepoGateOutput};

#[test]
fn run_repo_workspace_gates_executes_custom_malvin_checks_after_builtins() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_malvin_checks_line(work, "custom --option\n");
    let bin_dir = tempfile::tempdir().unwrap();
    let trace = bin_dir.path().join("trace.log");
    install_trace_echo_bins(bin_dir.path(), &trace, &["kiss", "custom"], 0);
    let _guard = set_fake_command_dir(bin_dir.path());
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = fs::read_to_string(&trace).unwrap();
    let kiss_check_pos = log.find("kiss check").expect("kiss check");
    let custom_pos = log.find("custom --option").expect("custom");
    assert!(
        kiss_check_pos < custom_pos,
        "built-ins should run before .malvin_checks lines"
    );
}

#[test]
fn run_repo_workspace_gates_does_not_create_malvin_checks() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_minimal_cargo_rs_py_tests(work);
    let malvin_checks = work.join(repo_gates::MALVIN_CHECKS_FILE);
    assert!(!malvin_checks.exists());
    let bin_dir = tempfile::tempdir().unwrap();
    let trace = bin_dir.path().join("trace.log");
    install_trace_echo_bins(bin_dir.path(), &trace, &["kiss", "ruff", "cargo"], 0);
    let _guard = set_fake_command_dir(bin_dir.path());
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    assert!(!malvin_checks.exists());
    let log = fs::read_to_string(&trace).unwrap();
    assert!(!log_contains_command(&log, "pre-commit run --all-files"));
    assert!(log_contains_command(&log, "kiss check"));
    assert!(log_contains_command(&log, "ruff check ."));
    assert!(log_contains_command(&log, "cargo clippy"));
    assert!(log_contains_command(&log, "cargo test"));
}

#[test]
fn run_repo_workspace_gates_runs_tree_builtins_without_git_or_malvin_checks() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    fs::write(
        work.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let trace = bin_dir.path().join("trace.log");
    install_trace_echo_bins(bin_dir.path(), &trace, &["kiss", "cargo"], 0);
    let _guard = set_fake_command_dir(bin_dir.path());
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = fs::read_to_string(&trace).unwrap();
    assert!(log_contains_command(&log, "kiss check"));
    assert!(log_contains_command(&log, "cargo clippy"));
    assert!(log_contains_command(&log, "cargo test"));
}

#[test]
fn run_repo_workspace_gates_skips_pytest_without_test_named_py_files() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    fs::create_dir(work.join(".git")).unwrap();
    fs::write(work.join("script.py"), "print('ok')\n").unwrap();
    let bin_dir = tempfile::tempdir().unwrap();
    let trace = bin_dir.path().join("trace.log");
    install_trace_echo_bins(bin_dir.path(), &trace, &["kiss", "ruff"], 0);
    let _guard = set_fake_command_dir(bin_dir.path());
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = fs::read_to_string(&trace).unwrap();
    assert!(log_contains_command(&log, "ruff check"));
    assert!(!log_contains_command(&log, "pytest -sv tests"));
}

#[test]
fn quality_checks_log_records_gate_lines_when_run_log_dir_set() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    let run_dir = work.join("malvin_run");
    fs::create_dir_all(&run_dir).unwrap();
    workspace_git_cargo_main_only(work);
    let bin_dir = tempfile::tempdir().unwrap();
    for name in ["kiss", "cargo"] {
        write_executable_script(bin_dir.path(), name, "#!/bin/sh\nexit 0\n");
    }
    let _guard = set_fake_command_dir(bin_dir.path());
    run_repo_workspace_gates(work, RepoGateOutput::Tagged, Some(&run_dir)).unwrap();
    let qlog = fs::read_to_string(run_dir.join("quality_checks.log")).unwrap();
    assert!(qlog.contains("Running `kiss check`"));
    assert!(qlog.contains("Running `cargo test`"));
}

#[test]
fn prepare_repo_workspace_skips_quality_commands() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_kissconfig_90_cargo_rs_py(work);
    let bin_dir = tempfile::tempdir().unwrap();
    let trace = bin_dir.path().join("trace.log");
    for name in ["kiss", "cargo", "ruff", "pytest"] {
        write_trace_echo_script(bin_dir.path(), name, &trace, 1);
    }
    let _guard = set_fake_command_dir(bin_dir.path());
    let result = prepare_repo_workspace(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    assert!(
        !trace.exists(),
        "workspace preparation must not run quality commands"
    );
}
