//! External kiss witnesses for `acp_session_tests` submodules (bucket D).

#[test]
fn kiss_witness_acp_session_test_fns() {
    let _ = stringify!(busy_session_with_dead_transport);
    let _ = stringify!(acp_session_cancel_clears_busy_state_after_rpc_error);
    let _ = stringify!(dead_transport_child_stdio);
    let _ = stringify!(dead_transport_sync_channels);
    let _ = stringify!(dead_transport_session_inner);
    let _ = stringify!(process_exists);
    let _ = stringify!(wait_for_pid_file);
    let _ = stringify!(write_descendant_spawning_acp_mock);
    let _ = stringify!(shutdown_kills_agent_spawned_descendants);
    let _ = stringify!(shutdown_sends_cancel_before_teardown);
    let _ = stringify!(spawn_descendant_mock_session);
    let _ = stringify!(assert_descendant_killed_after_shutdown);
}
