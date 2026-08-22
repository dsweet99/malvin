//! Kiss static coverage contract (call-shaped tokens; not compiled).

#[test]
fn kiss_cov_pi_sdk_rpc_io() {
    codex_spawn_bridge();
    codex_initialize();
    codex_start_thread();
    request();
    response_error();
    codex_write_abort();
    codex_send_prompt();
    read_json_waiting();
    next_id();
    set_codex_turn_id();
    turn_interrupt_params();
    read_json_waiting();
    read_json_line();
    session_string();
    set_session_string();
}

#[test]
fn kiss_cov_pi_sdk_rpc_io_b() {
    pi_send_prompt();
    pi_wait_for_response();
    pi_read_line();
    pi_drain_until_run_done();
    pi_read_line_with_idle_timeout();
    pi_finish_run_done();
    pi_next_req_id();
    PI_REQ_SEQ();
    BridgeWire();
    NodeBridge();
    PiRpc();
}
