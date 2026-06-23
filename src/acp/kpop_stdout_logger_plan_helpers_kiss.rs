//! External kiss witnesses for `kpop_stdout_logger_plan_helpers.rs`.

#[test]
fn kiss_cov_stdout_log_fixture_lifecycle() {
    let _guard = crate::acp_tests::kpop_stdout_logger_plan_helpers::stdout_log_test_guard();
    let fixture = crate::acp_tests::kpop_stdout_logger_plan_helpers::begin_stdout_log_fixture();
    let crate::acp_tests::kpop_stdout_logger_plan_helpers::StdoutLogFixture {
        tmp: _,
        stdout_path: _,
        trace_path: _,
    } = &fixture;
    assert!(fixture.stdout_path.to_string_lossy().contains("stdout"));
    let _ = crate::acp_tests::kpop_stdout_logger_plan_helpers::finish_stdout_log_fixture(fixture);
    let _ = crate::acp_tests::kpop_stdout_logger_plan_helpers::open_trace_writer;
    let _ = crate::acp_tests::kpop_stdout_logger_plan_helpers::open_styled_markdown_trace_writer;
    let _ = crate::acp_tests::kpop_stdout_logger_plan_helpers::styled_markdown_trace_writer;
    let _ = crate::acp_tests::kpop_stdout_logger_plan_helpers::tee_coalesced_update;
    let _ = crate::acp_tests::kpop_stdout_logger_plan_helpers::production_execute_done_stdout;
    let _ = crate::acp_tests::kpop_stdout_logger_plan_helpers::production_execute_done_trace_and_stdout;
}
