use std::fs;

use crate::test_utils::with_isolated_home;

use super::command_support::set_fake_command_dir;
use super::gate_run::prepare_repo_workspace;
use super::tests_gates_common::log_contains_command;
use super::tests_gates_helpers::{
    install_trace_echo_bins, workspace_git_cargo_main_only,
    workspace_git_kissconfig_90_cargo_rs_py, workspace_git_malvin_checks_line,
    write_executable_script, write_trace_echo_script,
};
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
    assert!(!log_contains_command(&log, "lint check"));
}

#[test]
fn run_repo_workspace_gates_errors_when_malvin_checks_missing() {
    with_isolated_home(|work| {
        super::tests_gates_helpers::git_init_work(work);
        fs::write(work.join("main.rs"), "fn main() {}\n").unwrap();
        fs::write(
            work.join(".kissconfig"),
            "[gate]\ntest_coverage_threshold = 90\n",
        )
        .unwrap();
        let malvin_checks = crate::malvin_checks_path(work);
        assert!(!malvin_checks.exists());
        let bin_dir = tempfile::tempdir().unwrap();
        write_executable_script(bin_dir.path(), "kiss", "#!/bin/sh\nexit 0\n");
        let _guard = set_fake_command_dir(bin_dir.path());
        let err = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None).unwrap_err();
        assert!(
            err.contains("missing"),
            "expected missing-checks error, got: {err}"
        );
        assert!(!malvin_checks.exists());
    });
}

#[test]
fn run_repo_workspace_gates_runs_seeded_kiss_only_without_git_or_malvin_checks() {
    with_isolated_home(|work| {
        fs::write(
            work.join("Cargo.toml"),
            "[package]\nname = 'm'\nversion = '0.1.0'\n",
        )
        .unwrap();
        super::tests_gates_helpers::seed_workspace_builtin_malvin_checks(work);
        let bin_dir = tempfile::tempdir().unwrap();
        let trace = bin_dir.path().join("trace.log");
        install_trace_echo_bins(bin_dir.path(), &trace, &["kiss"], 0);
        let _guard = set_fake_command_dir(bin_dir.path());
        let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
        assert!(result.is_ok());
        let log = fs::read_to_string(&trace).unwrap();
        assert!(log_contains_command(&log, "kiss check"));
        assert!(!log_contains_command(&log, "lint check"));
    });
}

#[test]
fn quality_gates_log_records_gate_lines_when_run_log_dir_set() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    let run_dir = work.join("malvin_run");
    fs::create_dir_all(&run_dir).unwrap();
    workspace_git_cargo_main_only(work);
    super::tests_gates_helpers::seed_workspace_builtin_malvin_checks(work);
    let bin_dir = tempfile::tempdir().unwrap();
    write_executable_script(
        bin_dir.path(),
        "kiss",
        "#!/bin/sh\necho \"stdout from $0\"\necho \"stderr from $0\" >&2\nexit 0\n",
    );
    let _guard = set_fake_command_dir(bin_dir.path());
    run_repo_workspace_gates(work, RepoGateOutput::Tagged, Some(&run_dir)).unwrap();
    let qlog = fs::read_to_string(run_dir.join("quality_gates.log")).unwrap();
    assert!(qlog.contains("Running `kiss check`"));
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
    for name in ["kiss", "lint", "gate_b"] {
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
