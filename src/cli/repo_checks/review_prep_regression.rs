use super::{RepoGateOutput, gate_run::emit_repo_gate_line};
use crate::test_stderr_capture::capture_stderr_output;

#[test]
fn repo_gate_stderr_output_must_match_malvin_log_format() {
    let msg = ".kissconfig gate.test_coverage_threshold is missing or below 90";
    let tmp = tempfile::tempdir().expect("tempdir");
    let stderr = capture_stderr_output(|| {
        emit_repo_gate_line(RepoGateOutput::Stderr, msg, Some(tmp.path()));
    });

    let log_path = tmp.path().join(crate::artifacts::QUALITY_GATES_LOG);
    let log = std::fs::read_to_string(&log_path).expect("quality_gates.log");
    assert!(
        log.contains("malvin") && log.contains(msg),
        "quality_gates.log must record malvin-formatted gate lines"
    );
    assert!(
        stderr.contains("malvin") && stderr.contains(msg),
        "repo gate Stderr output must reach stderr via print_stderr_line"
    );
}
