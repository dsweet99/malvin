use std::io::Write;

use super::{
    RepoGateOutput,
    gate_log::{emit_repo_gate_line, emit_repo_gate_warning},
    kissconfig_warn::warn_kissconfig_test_coverage_if_needed,
};
use crate::output::{format_who_tag_delim, MALVIN_WHO, WARNING_WHO};
use crate::test_stderr_capture::capture_stderr_output;

const KISSCONFIG_COVERAGE_WARN: &str = ".kissconfig gate.test_coverage_threshold is missing or below 90; editing code without sufficient unit test coverage is dangerous.";

#[test]
fn repo_gate_stderr_progress_must_use_malvin_who_not_warning() {
    let malvin_tag = format_who_tag_delim(MALVIN_WHO);
    let warning_tag = format_who_tag_delim(WARNING_WHO);
    let stderr = capture_stderr_output(|| {
        emit_repo_gate_line(RepoGateOutput::Stderr, "Running `kiss check`", None);
    });
    assert!(
        stderr.contains(&malvin_tag) && stderr.contains("kiss check"),
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
    let msg = KISSCONFIG_COVERAGE_WARN;
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
fn kissconfig_coverage_warn_must_use_malvin_who_on_stderr() {
    let malvin_tag = format_who_tag_delim(MALVIN_WHO);
    let warning_tag = format_who_tag_delim(WARNING_WHO);
    let tmp = tempfile::tempdir().expect("tempdir");
    let cfg = tmp.path().join(".kissconfig");
    let mut f = std::fs::File::create(&cfg).expect("create .kissconfig");
    writeln!(f, "[gate]").expect("write");
    writeln!(f, "test_coverage_threshold = 50").expect("write threshold");
    let stderr = capture_stderr_output(|| {
        warn_kissconfig_test_coverage_if_needed(tmp.path(), RepoGateOutput::Tagged, None);
    });
    assert!(
        stderr.contains(&malvin_tag) && stderr.contains("test_coverage_threshold"),
        "kissconfig coverage warnings must use malvin who on stderr, stderr={stderr:?}"
    );
    assert!(
        !stderr.contains(&warning_tag),
        "kissconfig coverage must not use warning who, stderr={stderr:?}"
    );
}

#[test]
fn repo_gate_stderr_output_must_match_malvin_log_format() {
    let malvin_tag = format_who_tag_delim(MALVIN_WHO);
    let msg = KISSCONFIG_COVERAGE_WARN;
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
