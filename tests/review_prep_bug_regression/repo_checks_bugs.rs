use super::helpers::manifest_root;

#[test]
fn quality_gates_log_test_fn_must_not_use_stale_quality_checks_name() {
    let src = std::fs::read_to_string(
        manifest_root().join("src/cli/repo_checks/tests_gates_unix_extra.rs"),
    )
    .expect("read tests_gates_unix_extra.rs");
    assert!(
        !src.contains("fn quality_checks_log_records_gate_lines_when_run_log_dir_set"),
        "bug: test fn name still says quality_checks_log after production switched to \
         quality_gates.log (QUALITY_GATES_LOG)"
    );
    assert!(
        src.contains("fn quality_gates_log_records_gate_lines_when_run_log_dir_set"),
        "bug: rename unix gate log test to quality_gates_log_records_gate_lines_when_run_log_dir_set"
    );
}
