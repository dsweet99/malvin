use super::{
    RepoGateOutput,
    gate_log::{emit_repo_gate_line, emit_repo_gate_warning},
};
use crate::output::{format_who_tag_delim, MALVIN_WHO, WARNING_WHO};
use crate::test_stderr_capture::capture_stderr_output;

const GATE_WARN_MSG: &str = "quality gate warning for regression test";

#[test]
fn repo_gate_stderr_progress_must_use_malvin_who_not_warning() {
    let malvin_tag = format_who_tag_delim(MALVIN_WHO);
    let warning_tag = format_who_tag_delim(WARNING_WHO);
    let stderr = capture_stderr_output(|| {
        emit_repo_gate_line(RepoGateOutput::Stderr, "Running `make lint`", None);
    });
    assert!(
        stderr.contains(&malvin_tag) && stderr.contains("make lint"),
        "gate progress on Stderr path must use malvin who on stderr, got: {stderr:?}"
    );
    assert!(
        !stderr.contains(&warning_tag),
        "gate progress must not use warning who tag, got: {stderr:?}"
    );
}

#[test]
fn quality_gates_log_stderr_gate_warning_must_use_malvin_who_tag() {
    let malvin_tag = format_who_tag_delim(MALVIN_WHO);
    let warning_tag = format_who_tag_delim(WARNING_WHO);
    let msg = GATE_WARN_MSG;
    let tmp = tempfile::tempdir().expect("tempdir");
    let stderr = capture_stderr_output(|| {
        emit_repo_gate_warning(msg, Some(tmp.path()));
    });
    assert!(
        stderr.contains(&malvin_tag) && !stderr.contains(&warning_tag),
        "stderr must use malvin who tag, got: {stderr:?}"
    );
    let log = std::fs::read_to_string(tmp.path().join(crate::artifacts::QUALITY_GATES_LOG))
        .expect("quality_gates.log");
    assert!(
        log.contains(&malvin_tag) && log.contains(msg),
        "quality_gates.log must record malvin who tag for gate warnings, got: {log:?}"
    );
}

#[test]
fn repo_gate_stderr_output_must_match_malvin_log_format() {
    let malvin_tag = format_who_tag_delim(MALVIN_WHO);
    let msg = GATE_WARN_MSG;
    let tmp = tempfile::tempdir().expect("tempdir");
    let stderr = capture_stderr_output(|| {
        emit_repo_gate_warning(msg, Some(tmp.path()));
    });

    let log_path = tmp.path().join(crate::artifacts::QUALITY_GATES_LOG);
    let log = std::fs::read_to_string(&log_path).expect("quality_gates.log");
    assert!(
        log.contains(&malvin_tag) && log.contains(msg),
        "quality_gates.log must record malvin who tag for gate warnings"
    );
    assert!(
        stderr.contains(&malvin_tag) && stderr.contains(msg),
        "gate warnings must reach stderr via print_stderr_line(malvin)"
    );
}
