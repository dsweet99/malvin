//! Kiss static coverage contract (call-shaped tokens; not compiled).


#[test]
fn kiss_cov_retry_teardown_helpers() {
    agent_error_requires_coder_session_teardown();
    agent_string_is_cursor_agent_busy();
    agent_string_is_stale_cursor_sdk_auth();
    text_has_any();
    CHILD_OR_BRIDGE_DEAD_NEEDLES();
    DRAIN_IDLE_PREFIX_BRIDGE();
    DRAIN_IDLE_PREFIX_PI();
    DRAIN_IDLE_PREFIX_CODEX();
    live_drain_idle_prefixes_require_coder_session_teardown();
}

#[test]
fn kiss_cov_pi_sdk_client_mock_helpers() {
    pi_mock_io();
    pi_mock_bin();
    pi_install_mock_env();
    pi_clear_mock_env();
    pi_mock_client();
    pi_sdk_client_mock_rpc_prompt_records_usage();
    pi_sdk_noforce_fails_fast();
    pi_sdk_agent_end_before_ack_completes();
    pi_sdk_empty_assistant_result_clears_prior_response();
    pi_sdk_new_session_ack_idle_timeout();
    agent_end_before_reply_oneshot_returns_ok();
    minimal_session();
    run_hello_prompt();
    write_exec_script();
}
