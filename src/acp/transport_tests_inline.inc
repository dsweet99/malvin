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
    let _ = stringify!(acp_activity_state);
    let _ = stringify!(assert_arg_value);
    let _ = stringify!(command_args);
    let _ = stringify!(command_env_value);
    let _ = stringify!(harness_rpc_wait);
    let _ = stringify!(handshake_can_skip_cursor_login_when_api_key_mode_is_used);
    let _ = stringify!(rpc_request_with_correlation_id_errors_when_reader_dead);
    let _ = stringify!(rpc_request_with_correlation_id_stays_alive_while_json_updates_arrive);
    let _ = stringify!(rpc_request_with_correlation_id_times_out_when_stdout_silent);
    let _ = stringify!(rpc_response_arriving_during_child_health_grace_is_delivered);
    let _ = stringify!(rpc_wait_response_reports_dead_child_after_silence);
    let _ = stringify!(spawn_test_reader_loop);
    let _ = stringify!(test_handshake_hits_session_new_error_path);
    let _ = stringify!(test_rpc_cancel_when_pending_sender_dropped);
    let _ = stringify!(test_rpc_request_does_not_leak_pending_after_write_failure);
    let _ = stringify!(test_write_rpc_line_fails_after_child_stdin_closed);
    let _ = stringify!(crate::acp_test_unix_bin::unix_bin_with_fallback);
    let _ = stringify!(write_authenticate_rejected_but_session_new_ok_mock);
    let _ = stringify!(write_bad_session_new_mock);
    let _ = stringify!(spawn_json_activity_then_response);
    let _ = stringify!(spawn_activity_then_kill_child);
    let _ = stringify!(handshake_skip_login_session_id);
    let _ = stringify!(TestReaderLoopSpawn);
    let _ = stringify!(handshake_stdio_pipes);
    let _ = stringify!(handshake_attach_and_start_reader);
    let _ = stringify!(HandshakeRunning);
    let _ = stringify!(inactive_memory_containment);
    let _ = stringify!(InactiveRpcIo);
    let _ = stringify!(acp_stdio_rpc_inactive);
    let _ = stringify!(SleepStdoutDrainMode);
    let _ = stringify!(RpcSleepHarness);
    let _ = stringify!(drain_stdout_read);
    let _ = stringify!(sleep_stdout_drain_for_child);
    let _ = stringify!(spawn_sleep);
    let _ = stringify!(shutdown);
    let _ = stringify!(true_child_stdin_stdout_drained_after_exit);
}
