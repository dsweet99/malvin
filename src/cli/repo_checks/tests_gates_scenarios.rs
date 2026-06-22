//! Gate-schedule scenario bodies shared by `repo_checks` unit tests.

use std::fs;

use crate::repo_gates;

use super::gate_schedule_recorder::{
    arm_gate_command_recorder, arm_gate_command_recorder_with_echo_streams,
    recordings_as_gate_trace_log, take_recorded_gate_commands,
};
use super::gate_run::{
    prepare_repo_workspace, run_repo_workspace_gates, run_repo_workspace_gates_no_kiss_clamp_with_details,
    run_repo_workspace_gates_with_details,
};
use super::tests_gates_common::log_contains_command;
use super::tests_gates_helpers::{
    workspace_git_cargo_main_only, workspace_git_kissconfig_90_cargo_rs_py,
    workspace_git_malvin_checks_line, workspace_git_minimal_cargo_rs_py_tests,
    workspace_git_precommit_malvin_checks_cargo_main,
};
use super::RepoGateOutput;

pub(super) fn scenario_invokes_expected_quality_commands() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_minimal_cargo_rs_py_tests(work);
    let _guard = arm_gate_command_recorder();
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = recordings_as_gate_trace_log(&take_recorded_gate_commands());
    assert!(log.contains("kiss clamp"));
    assert!(log.contains("kiss check"));
    assert!(log.contains("cargo clippy"));
    assert!(log.contains(repo_gates::default_rust_test_command(work)));
    assert!(log_contains_command(&log, "ruff check"));
    assert!(!log_contains_command(&log, "pytest"));
}

pub(super) fn scenario_skips_pre_commit_when_config_present() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_precommit_malvin_checks_cargo_main(work);
    let _guard = arm_gate_command_recorder();
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = recordings_as_gate_trace_log(&take_recorded_gate_commands());
    assert!(!log_contains_command(&log, "pre-commit run --all-files"));
    assert!(!log_contains_command(&log, "kiss check"));
    assert!(!log_contains_command(&log, "cargo clippy"));
    assert!(log_contains_command(&log, "custom --only"));
}

pub(super) fn scenario_executes_only_malvin_checks_when_present() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_malvin_checks_line(work, "custom --option\n");
    let _guard = arm_gate_command_recorder();
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = recordings_as_gate_trace_log(&take_recorded_gate_commands());
    assert!(log_contains_command(&log, "custom --option"));
    assert!(!log_contains_command(&log, "kiss check"));
    assert!(!log_contains_command(&log, "cargo clippy"));
}

pub(super) fn scenario_materializes_default_malvin_checks() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    fs::create_dir(work.join(".git")).unwrap();
    fs::write(work.join("main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        work.join(".kissconfig"),
        "[gate]\ntest_coverage_threshold = 90\n",
    )
    .unwrap();
    let malvin_checks = work.join(repo_gates::MALVIN_CHECKS_FILE);
    assert!(!malvin_checks.exists());
    let _guard = arm_gate_command_recorder();
    assert!(run_repo_workspace_gates(work, RepoGateOutput::Tagged, None).is_ok());
    assert!(
        !malvin_checks.exists(),
        "ephemeral gate runs must restore Missing .malvin/checks so repo-root shadow files \
         are not left behind"
    );
}

pub(super) fn scenario_runs_tree_builtins_without_git_or_malvin_checks() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    fs::write(
        work.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .unwrap();
    let _guard = arm_gate_command_recorder();
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = recordings_as_gate_trace_log(&take_recorded_gate_commands());
    assert!(log_contains_command(&log, "kiss check"));
    assert!(log_contains_command(&log, "cargo clippy"));
    assert!(log_contains_command(&log, repo_gates::DEFAULT_RUST_NEXTEST_PARTITION_1));
}

pub(super) fn scenario_skips_pytest_without_test_named_py_files() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    fs::create_dir(work.join(".git")).unwrap();
    fs::write(work.join("script.py"), "print('ok')\n").unwrap();
    let _guard = arm_gate_command_recorder();
    let result = run_repo_workspace_gates(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = recordings_as_gate_trace_log(&take_recorded_gate_commands());
    assert!(log_contains_command(&log, "ruff check"));
    assert!(!log_contains_command(&log, "pytest -sv tests"));
}

pub(super) fn scenario_quality_gates_log_records_gate_lines_when_run_log_dir_set() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    let run_dir = work.join("malvin_run");
    fs::create_dir_all(&run_dir).unwrap();
    workspace_git_cargo_main_only(work);
    let _guard = arm_gate_command_recorder_with_echo_streams();
    run_repo_workspace_gates(work, RepoGateOutput::Tagged, Some(&run_dir)).unwrap();
    let qlog = fs::read_to_string(run_dir.join("quality_gates.log")).unwrap();
    assert!(qlog.contains("Running `kiss check`"));
    assert!(qlog.contains(&format!(
        "Running `{}`",
        repo_gates::DEFAULT_RUST_NEXTEST_PARTITION_1
    )));
    assert!(qlog.contains(&format!(
        "Running `{}`",
        repo_gates::DEFAULT_RUST_NEXTEST_PARTITION_2
    )));
    assert!(qlog.contains("[stdout]"));
    assert!(qlog.contains("[stderr]"));
    assert!(qlog.contains("stdout from"));
    assert!(qlog.contains("stderr from"));
}

pub(super) fn scenario_prepare_repo_workspace_skips_quality_commands() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_kissconfig_90_cargo_rs_py(work);
    let _guard = arm_gate_command_recorder();
    let result = prepare_repo_workspace(work, RepoGateOutput::Tagged, None);
    assert!(result.is_ok());
    let log = recordings_as_gate_trace_log(&take_recorded_gate_commands());
    assert!(
        !log.contains("kiss clamp"),
        "existing .kissconfig must not trigger prep clamp: {log}"
    );
    assert!(!log_contains_command(&log, "kiss check"));
    assert!(!log_contains_command(&log, "cargo clippy"));
}

pub(super) fn scenario_strict_kissconfig_full_gates_skips_auto_clamp_before_kiss_check() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_kissconfig_90_cargo_rs_py(work);
    let _guard = arm_gate_command_recorder();
    run_repo_workspace_gates(work, RepoGateOutput::Tagged, None).expect("gates");
    let log = recordings_as_gate_trace_log(&take_recorded_gate_commands());
    assert!(
        !log.contains("kiss clamp"),
        "threshold-90 .kissconfig must not trigger auto clamp before kiss check: {log}"
    );
    assert!(
        log_contains_command(&log, "kiss check"),
        "default builtin schedule must still run kiss check: {log}"
    );
}

pub(super) fn scenario_runs_kiss_clamp_from_checks_when_kissconfig_valid() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_kissconfig_90_cargo_rs_py(work);
    fs::create_dir_all(work.join(".malvin")).unwrap();
    fs::write(work.join(".malvin/checks"), "kiss clamp\nkiss check\n").unwrap();
    let _guard = arm_gate_command_recorder();
    run_repo_workspace_gates(work, RepoGateOutput::Tagged, None).expect("gates");
    let log = recordings_as_gate_trace_log(&take_recorded_gate_commands());
    assert!(
        log.contains("kiss clamp"),
        "explicit kiss clamp in .malvin/checks must run even when .kissconfig is parseable: {log}"
    );
    assert!(log_contains_command(&log, "kiss check"));
}

pub(super) fn scenario_runs_kiss_clamp_when_no_kissconfig() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    fs::write(work.join("main.rs"), "fn main() {}\n").unwrap();
    let _guard = arm_gate_command_recorder();
    prepare_repo_workspace(work, RepoGateOutput::Tagged, None).expect("prepare");
    let log = recordings_as_gate_trace_log(&take_recorded_gate_commands());
    assert!(log.contains("kiss clamp"));
}

pub(super) fn scenario_gate_run_wires_quality_gates_runner() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    fs::write(
        work.join("Cargo.toml"),
        "[package]\nname = \"m\"\nversion = \"0.1.0\"\n",
    )
    .expect("Cargo.toml");
    let _guard = arm_gate_command_recorder();
    run_repo_workspace_gates_with_details(work, RepoGateOutput::Tagged, None).expect("quality gates");
}

pub(super) fn scenario_quality_gates_with_details_skips_auto_clamp_before_kiss_check() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    workspace_git_kissconfig_90_cargo_rs_py(work);
    fs::create_dir_all(work.join(".malvin")).unwrap();
    fs::write(work.join(".malvin/checks"), "kiss check\n").unwrap();
    let _guard = arm_gate_command_recorder();
    run_repo_workspace_gates_with_details(work, RepoGateOutput::Tagged, None).expect("quality gates");
    let log = recordings_as_gate_trace_log(&take_recorded_gate_commands());
    assert!(
        !log.contains("kiss clamp"),
        "run_repo_workspace_gates_with_details must not auto-clamp threshold-90 .kissconfig before kiss check: {log}"
    );
    assert!(log_contains_command(&log, "kiss check"));
}

pub(super) fn scenario_gate_run_wires_workspace_gates_runner() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    fs::write(
        work.join("Cargo.toml"),
        "[package]\nname = \"m\"\nversion = \"0.1.0\"\n",
    )
    .expect("Cargo.toml");
    let _guard = arm_gate_command_recorder();
    run_repo_workspace_gates_with_details(work, RepoGateOutput::Tagged, None)
        .expect("workspace gates");
}

pub(super) fn scenario_gate_run_wires_no_kiss_clamp_runner() {
    let tmp = tempfile::tempdir().unwrap();
    let work = tmp.path();
    fs::write(
        work.join("Cargo.toml"),
        "[package]\nname = \"m\"\nversion = \"0.1.0\"\n",
    )
    .expect("Cargo.toml");
    let _guard = arm_gate_command_recorder();
    run_repo_workspace_gates_no_kiss_clamp_with_details(work, RepoGateOutput::Tagged, None)
        .expect("workspace gates without kiss clamp");
}
