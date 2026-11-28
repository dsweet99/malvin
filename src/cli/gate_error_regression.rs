use crate::cli::CliError;
use crate::cli::repo_checks::{GateRunError, RepoGateCommandFailure, RepoGateFailure};
use malvin::output::{ERROR_WHO, format_who_tag_delim};
use malvin::test_support::test_stderr_capture::capture_stderr_output;

use super::entrypoint_support::print_cli_error;

fn sample_command_failure() -> RepoGateCommandFailure {
    RepoGateCommandFailure {
        command: "make lint".to_string(),
        exit_code: Some(1),
        stdout: "gate-out".to_string(),
        stderr: "gate-err".to_string(),
    }
}

fn router_gate_error() -> CliError {
    let failure = GateRunError::Gate(RepoGateFailure::Command(sample_command_failure()));
    CliError::from_router_gates("malvin", &failure)
}

#[test]
fn a_failed_gate_command_is_a_gate_failure_and_a_setup_error_is_not() {
    assert!(matches!(router_gate_error(), CliError::GatesFailed(_)));
    let setup = GateRunError::Setup("restore .malvin/gates: blocked".to_string());
    assert!(matches!(
        CliError::from_router_gates("malvin", &setup),
        CliError::Other(_)
    ));
    assert!(matches!(
        CliError::from("text that mentions failed (exit 1)"),
        CliError::Other(_)
    ));
}

#[test]
fn printing_a_gate_failure_avoids_the_error_tag_and_keeps_context() {
    let error = router_gate_error();
    let error_tag = format_who_tag_delim(ERROR_WHO);
    let stderr = capture_stderr_output(|| print_cli_error(&error));
    assert!(!stderr.contains(&error_tag), "{stderr:?}");
    assert!(
        stderr.contains("Workspace checks did not pass"),
        "{stderr:?}"
    );
    assert_eq!(
        stderr.matches("`make lint` failed (exit 1)").count(),
        1,
        "{stderr:?}"
    );
}

#[test]
fn a_gate_failure_is_not_written_to_the_run_error_log() {
    let _guard = malvin::workspace::run_id::ACTIVE_RUN_DIR_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let dir = tempfile::tempdir().expect("temp");
    malvin::workspace::run_id::activate_run(dir.path().to_path_buf());
    let _ = capture_stderr_output(|| print_cli_error(&router_gate_error()));
    let _ = capture_stderr_output(|| print_cli_error(&CliError::from("plain failure")));
    super::error_run_log::clear_command_error_run_dir();
    let log = std::fs::read_to_string(dir.path().join("malvin_error.log")).unwrap_or_default();
    assert!(log.contains("plain failure"), "{log:?}");
    assert!(!log.contains("make lint"), "{log:?}");
}

#[test]
fn the_gate_report_prints_each_stream_once() {
    let failure = RepoGateFailure::Command(sample_command_failure());
    let stderr = capture_stderr_output(|| failure.emit_repo_gate_failure_stderr());
    assert_eq!(stderr.matches("stdout:").count(), 1, "{stderr:?}");
    assert_eq!(stderr.matches("gate-out").count(), 1, "{stderr:?}");
}
