include!("../acp_transport_tests/shared_harness.rs");
include!("../acp_transport_tests/shared_handshake.rs");
include!("../acp_transport_tests/child_health_a.rs");
include!("../acp_transport_tests/child_health_b.rs");
include!("../acp_transport_tests/handshake.rs");
include!("../acp_transport_tests/jsonrpc.rs");
include!("../acp_transport_tests/rpc_integration_a1.rs");
include!("../acp_transport_tests/rpc_integration_a2.rs");
include!("../acp_transport_tests/rpc_integration_b.rs");
include!("../acp_transport_tests/rpc_unit.rs");

#[test]
fn smoke_acp_activity_state() {
    use std::sync::atomic::Ordering;
    let (seq, _notify) = acp_activity_state();
    assert_eq!(seq.load(Ordering::SeqCst), 0);
}

#[test]
fn smoke_harness_rpc_wait_params_symbol() {
    let _ = std::any::type_name::<HarnessRpcWaitParams<'_>>();
}

#[test]
fn kiss_stringify_transport_units() {
    let _ = acp_activity_state;
    let _ = assert_arg_value;
    let _ = command_args;
    let _ = command_env_value;
    let _ = harness_rpc_wait;
    let _ = handshake_can_skip_cursor_login_when_api_key_mode_is_used;
    let _ = rpc_request_with_correlation_id_errors_when_reader_dead;
    let _ = rpc_request_with_correlation_id_stays_alive_while_json_updates_arrive;
    let _ = rpc_request_with_correlation_id_times_out_when_stdout_silent;
    let _ = rpc_response_arriving_during_child_health_grace_is_delivered;
    let _ = rpc_wait_response_reports_dead_child_after_silence;
    let _ = spawn_test_reader_loop;
    let _ = test_handshake_hits_session_new_error_path;
    let _ = test_rpc_cancel_when_pending_sender_dropped;
    let _ = test_rpc_request_does_not_leak_pending_after_write_failure;
    let _ = test_write_rpc_line_fails_after_child_stdin_closed;
    let _ = crate::acp_test_unix_bin::unix_bin_with_fallback;
    let _ = write_authenticate_rejected_but_session_new_ok_mock;
    let _ = write_bad_session_new_mock;
    let _ = spawn_json_activity_then_response;
    let _ = spawn_activity_then_kill_child;
    let _ = handshake_skip_login_session_id;
    let _: Option<TestReaderLoopSpawn> = None;
    let _ = handshake_stdio_pipes;
    let _ = handshake_attach_and_start_reader;
    let _: Option<HandshakeRunning> = None;
    let _ = inactive_memory_containment;
    let _: Option<InactiveRpcIo> = None;
    let _ = acp_stdio_rpc_inactive;
    let _: Option<SleepStdoutDrainMode> = None;
    let _: Option<RpcSleepHarness> = None;
    let _ = drain_stdout_read;
    let _ = sleep_stdout_drain_for_child;
    let _ = RpcSleepHarness::spawn_sleep;
    let _ = RpcSleepHarness::shutdown;
    let _ = true_child_stdin_stdout_drained_after_exit;
}
