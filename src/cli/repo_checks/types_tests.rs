use super::{GateRunError, RepoGateCommandFailure, RepoGateFailure, RepoGateOutput};

fn command_failure(exit_code: Option<i32>) -> RepoGateFailure {
    RepoGateFailure::Command(RepoGateCommandFailure {
        command: "make lint".to_string(),
        exit_code,
        stdout: "out".to_string(),
        stderr: "err".to_string(),
    })
}

#[test]
fn a_command_failure_displays_its_command_and_exit_without_output() {
    assert_eq!(command_failure(Some(1)).to_string(), "`make lint` failed (exit 1)");
    assert_eq!(
        command_failure(None).to_string(),
        "`make lint` failed (exit signal)"
    );
    let message = RepoGateFailure::Message("`x` failed to start: nope".to_string());
    assert_eq!(message.to_string(), "`x` failed to start: nope");
    let _: RepoGateOutput = RepoGateOutput::Tagged;
}

#[test]
fn only_a_failed_gate_command_counts_as_gates_having_run() {
    assert!(GateRunError::Gate(command_failure(Some(2))).is_command_failure());
    assert!(!GateRunError::Gate(RepoGateFailure::Message("no start".into())).is_command_failure());
    assert!(!GateRunError::from("restore failed".to_string()).is_command_failure());
}

#[test]
fn stderr_report_lists_the_summary_then_both_streams() {
    let lines = malvin::test_support::test_stderr_capture::capture_stderr_output(|| {
        command_failure(Some(1)).emit_repo_gate_failure_stderr();
    });
    let summary = lines.find("`make lint` failed (exit 1)").expect("summary");
    let out = lines.find("stdout:").expect("stdout label");
    let err = lines.find("stderr:").expect("stderr label");
    assert!(summary < out && out < err, "{lines}");
}
