use std::fs;

use crate::repo_gates;

use super::command_support::set_fake_command_dir;
use super::tests_gates_common::log_contains_command;
use super::tests_gates_helpers::{
    install_trace_echo_bins, workspace_git_cargo_main_only,
    workspace_git_kissconfig_90_cargo_rs_py, workspace_git_malvin_checks_line,
    workspace_git_minimal_cargo_rs_py_tests, write_executable_script, write_trace_echo_script,
};
use super::gate_run::prepare_repo_workspace;
use super::{RepoGateOutput, run_repo_workspace_gates};

#[test]
fn run_repo_workspace_gates_executes_only_malvin_checks_when_present() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_malvin_checks_line(work, "custom --option\n");
    let bin_dir = tempfile::tempdir().unwrap();
    let trace = bin_dir.path().join("trace.log");
    install_trace_echo_bins(bin_dir.path(), &trace, &["custom"], 0);
    let _guard = set_fake_command_dir(bin_dir.path());
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = fs::read_to_string(&trace).unwrap();
    assert!(log_contains_command(&log, "custom --option"));
    assert!(!log_contains_command(&log, "kiss check"));
    assert!(!log_contains_command(&log, "cargo clippy"));
}

fn assert_default_gate_trace(log: &str) {
    assert!(!log_contains_command(log, "pre-commit run --all-files"));
    assert!(log_contains_command(log, "kiss check"));
    assert!(log_contains_command(log, "ruff check ."));
    assert!(log_contains_command(log, "cargo clippy"));
    assert!(log_contains_command(log, "cargo test"));
}

fn materialize_default_checks_fixture() -> (tempfile::TempDir, std::path::PathBuf, Vec<String>) {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path().to_path_buf();
    workspace_git_minimal_cargo_rs_py_tests(&work);
    let expected = repo_gates::gate_command_lines(&work).unwrap();
    (tmp, work, expected)
}

#[test]
fn run_repo_workspace_gates_materializes_default_malvin_checks() {
    let (_tmp, work, expected) = materialize_default_checks_fixture();
    let malvin_checks = work.join(repo_gates::MALVIN_CHECKS_FILE);
    assert!(!malvin_checks.exists());
    let bin_dir = tempfile::tempdir().unwrap();
    let trace = bin_dir.path().join("trace.log");
    install_trace_echo_bins(bin_dir.path(), &trace, &["kiss", "ruff", "cargo"], 0);
    let _guard = set_fake_command_dir(bin_dir.path());
    assert!(run_repo_workspace_gates(&work, RepoGateOutput::Tagged, None).is_ok());
    assert!(
        !malvin_checks.exists(),
        "ephemeral gate runs must restore Missing .malvin_checks so repo-root shadow files \
         are not left behind"
    );
    std::fs::write(&malvin_checks, expected.join("\n") + "\n").unwrap();
    assert_eq!(repo_gates::load_malvin_checks(&malvin_checks).unwrap(), expected);
    assert_default_gate_trace(&fs::read_to_string(&trace).unwrap());
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
fn quality_gates_log_records_gate_lines_when_run_log_dir_set() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    let run_dir = work.join("malvin_run");
    fs::create_dir_all(&run_dir).unwrap();
    workspace_git_cargo_main_only(work);
    let bin_dir = tempfile::tempdir().unwrap();
    for name in ["kiss", "cargo"] {
        write_executable_script(
            bin_dir.path(),
            name,
            "#!/bin/sh\necho \"stdout from $0\"\necho \"stderr from $0\" >&2\nexit 0\n",
        );
    }
    let _guard = set_fake_command_dir(bin_dir.path());
    run_repo_workspace_gates(work, RepoGateOutput::Tagged, Some(&run_dir)).unwrap();
    let qlog = fs::read_to_string(run_dir.join("quality_gates.log")).unwrap();
    assert!(qlog.contains("Running `kiss check`"));
    assert!(qlog.contains("Running `cargo test`"));
    assert!(qlog.contains("[stdout]"));
    assert!(qlog.contains("[stderr]"));
    assert!(qlog.contains("stdout from"));
    assert!(qlog.contains("stderr from"));
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
