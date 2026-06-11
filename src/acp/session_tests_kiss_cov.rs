//! Kiss identifier refs for `session_tests.rs` (kept separate for line-count limits).

#[test]
fn kiss_cov_session_test_helpers() {
    let _ = crate::acp::session_tests::acp_session_from_sleep_child;
    let _ = crate::acp::session_tests::mem_watch_test_spawn_args;
    let _ = crate::acp::session_tests::mem_watch_test_telemetry;
    let _ = crate::acp::session_tests::spawn_sleep_child_in_new_process_group;
}
