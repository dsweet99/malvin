use super::*;
use std::fs;

#[test]
fn gate_command_lines_errors_when_malvin_checks_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    let err = gate_command_lines(w).unwrap_err();
    assert!(err.contains("is missing"), "unexpected error: {err}");
}

#[test]
fn command_matches_malvin_checks_gate_uses_checks_file_not_hardcoded_needles() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::create_dir_all(w.join(".malvin")).unwrap();
    fs::write(w.join(".malvin/checks"), "custom-gate --flag\n").unwrap();
    assert!(command_matches_malvin_checks_gate("sh -c custom-gate --flag", w));
    assert!(!command_matches_malvin_checks_gate("cargo nextest run", w));
}

#[test]
fn command_matches_malvin_checks_gate_sees_sandbox_expanded_nextest() {
    let tmp = tempfile::tempdir().unwrap();
    let w = tmp.path();
    fs::create_dir_all(w.join(".malvin")).unwrap();
    fs::write(w.join(".malvin/checks"), "cargo nextest run\n").unwrap();
    assert!(command_matches_malvin_checks_gate(
        DEFAULT_RUST_NEXTEST_PARTITION_1,
        w
    ));
}
