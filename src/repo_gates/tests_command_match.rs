use super::*;

#[test]
fn gate_command_lines_errors_when_malvin_checks_missing() {
    crate::test_utils::with_isolated_home(|w| {
        let err = gate_command_lines(w).unwrap_err();
        assert!(err.contains("is missing"), "unexpected error: {err}");
    });
}

#[test]
fn command_matches_malvin_checks_gate_uses_checks_file_not_hardcoded_needles() {
    crate::test_utils::with_isolated_home(|w| {
        std::fs::create_dir_all(w.join(".malvin")).unwrap();
        std::fs::write(w.join(".malvin/checks"), "custom-gate --flag\n").unwrap();
        assert!(command_matches_malvin_checks_gate("sh -c custom-gate --flag", w));
        assert!(!command_matches_malvin_checks_gate("cargo nextest run", w));
    });
}

#[test]
fn command_matches_malvin_checks_gate_sees_sandbox_expanded_nextest() {
    crate::test_utils::with_isolated_home(|w| {
        std::fs::create_dir_all(w.join(".malvin")).unwrap();
        std::fs::write(w.join(".malvin/checks"), "cargo nextest run\n").unwrap();
        assert!(command_matches_malvin_checks_gate(
            DEFAULT_RUST_NEXTEST_PARTITION_1,
            w
        ));
    });
}
