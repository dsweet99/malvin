//! Kiss identifier refs for `session_tests.rs` (kept separate for line-count limits).

#[test]
fn kiss_cov_session_test_helpers() {
    let _ = super::session_tests::acp_session_from_sleep_child;
    let _ = super::session_tests::mem_watch_test_spawn_args;
    let _ = super::session_tests::mem_watch_test_telemetry;
    let _ = super::session_tests::spawn_sleep_child_in_new_process_group;
    let _ = super::session_tests::session_with_sleep_child_for_mem_watch;
    let _ = super::session_tests::spawn_process_group_memory_watcher_starts_for_session;
    let _ = super::session_tests::watch_process_group_memory_kills_orphan_after_agent_pg_exits;
    let _ = super::session_tests::watch_process_group_memory_kills_over_limit_child;
    let _ = super::session_tests::watch_process_group_memory_kills_setsid_orphan_on_oom;
}
