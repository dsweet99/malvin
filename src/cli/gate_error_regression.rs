use crate::repo_checks::{
    GATE_FAILURE_MARKER, RepoGateCommandFailure, RepoGateFailure, is_gate_failure_error,
    repo_gate_failure_to_string,
};
use crate::test_stderr_capture::capture_stderr_output;

use super::entrypoint::print_command_error;

fn sample_command_failure() -> RepoGateCommandFailure {
    RepoGateCommandFailure {
        command: "kiss check".to_string(),
        exit_code: Some(1),
        stdout: "gate-out".to_string(),
        stderr: "gate-err".to_string(),
    }
}

#[test]
fn wrapped_post_run_retry_error_is_classified_as_gate_failure() {
    let gate_err = repo_gate_failure_to_string(RepoGateFailure::Command(sample_command_failure()));
    let wrapped = format!("post-run gates still failing after tidy kpop session: {gate_err}");
    assert!(
        is_gate_failure_error(&wrapped),
        "tidy-retry wrapper must not break gate-failure detection; got: {wrapped:?}"
    );
}

#[test]
fn print_command_error_on_wrapped_gate_retry_avoids_error_tag() {
    let wrapped = format!(
        "post-run gates still failing after tidy kpop session: {}",
        repo_gate_failure_to_string(RepoGateFailure::Command(sample_command_failure()))
    );
    let stderr = capture_stderr_output(|| print_command_error(&wrapped));
    assert!(
        !stderr.contains("[error"),
        "wrapped gate failures must not be painted as [error]; got: {stderr:?}"
    );
    assert!(
        !stderr.contains(GATE_FAILURE_MARKER),
        "internal gate marker must not be user-facing; got: {stderr:?}"
    );
    assert!(
        stderr.contains("post-run gates still failing after tidy kpop session"),
        "wrapped context should remain visible; got: {stderr:?}"
    );
    assert!(
        stderr.contains("`kiss check` failed (exit 1)"),
        "gate summary should remain visible; got: {stderr:?}"
    );
}

#[test]
fn gate_failure_entrypoint_path_prints_summary_once() {
    let summary = "`kiss check` failed (exit 1)";
    let stderr = capture_stderr_output(|| {
        let msg = repo_gate_failure_to_string(RepoGateFailure::Command(sample_command_failure()));
        assert!(is_gate_failure_error(&msg));
        print_command_error(&msg);
    });
    let count = stderr.matches(summary).count();
    assert_eq!(
        count, 1,
        "summary must appear once (emit + entrypoint currently duplicate); stderr={stderr:?}"
    );
}

#[test]
fn repeated_gate_failure_conversion_emits_body_once() {
    let failure = sample_command_failure();
    let stderr = capture_stderr_output(|| {
        let _ = repo_gate_failure_to_string(RepoGateFailure::Command(failure));
    });
    assert_eq!(
        stderr.matches("stdout:").count(),
        1,
        "gate body stdout section must not be duplicated; stderr={stderr:?}"
    );
    assert_eq!(
        stderr.matches("gate-out").count(),
        1,
        "gate stdout payload must not be duplicated; stderr={stderr:?}"
    );
}
