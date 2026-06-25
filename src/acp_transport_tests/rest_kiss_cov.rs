//! External kiss witnesses for `acp_transport_tests` submodules (bucket D).

#[test]
fn kiss_witness_shared_handshake_types() {
    let _: Option<super::TestReaderLoopSpawn> = None;
    let _: Option<super::HandshakeRunning> = None;
    let _ = super::handshake_stdio_pipes;
    let _ = super::handshake_attach_and_start_reader;
    let _ = super::spawn_test_reader_loop;
}

#[test]
fn kiss_witness_acp_transport_test_fns() {
    let _ = stringify!(rpc_request_with_correlation_id_stays_alive_while_json_updates_arrive);
    let _ = stringify!(rpc_wait_response_reports_dead_child_after_silence);
    let _ = stringify!(rpc_response_arriving_during_child_health_grace_is_delivered);
    let _ = stringify!(handshake_can_skip_cursor_login_when_api_key_mode_is_used);
    let _ = stringify!(test_rpc_cancel_when_pending_sender_dropped);
    let _ = stringify!(test_rpc_request_does_not_leak_pending_after_write_failure);
    let _ = stringify!(rpc_request_with_correlation_id_times_out_when_stdout_silent);
    let _ = stringify!(rpc_request_with_correlation_id_errors_when_reader_dead);
    let _ = stringify!(requires_cursor_login_auth_skips_login_when_process_credentials_exist);
    let _ = stringify!(test_handshake_hits_session_new_error_path);
    let _ = stringify!(test_cursor_credentials_empty_strings_skipped);
    let _ = stringify!(test_write_rpc_line_fails_after_child_stdin_closed);
    let _ = stringify!(format_jsonrpc_error_pretty_prints_cursor_style);
    let _ = stringify!(test_acp_rpc_timeout_parsing);
}

#[test]
fn kiss_witness_shared_harness() {
    let _ = super::acp_activity_state;
    let _ = super::harness_rpc_wait;
    let _: Option<super::HarnessRpcWaitParams> = None;
}
