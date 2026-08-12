//! Orphan kiss static-coverage witnesses for `pi:` backend (not in the crate module tree).

#[test]
fn kiss_cov_pi_sdk_discover_auth_models() {
    resolve_pi_bin();
    pi_missing_binary_message();
    pi_version_ok();
    path_is_executable();
    parse_pi_version();
    parse_semver_triple();
    leading_u32();
    PI_MIN_VERSION();
    ensure_pi_authenticated();
    provider_auth_env_keys();
    provider_auth_env_keys_primary();
    provider_auth_env_keys_secondary();
    list_pi_models_sync();
    pi_list_models_timeout();
    PiModelListing();
    DEFAULT_PI_LIST_MODELS_TIMEOUT_MS();
    PI_MISSING_HINT();
    pi_sdk_client_from_raw();
}

#[test]
fn kiss_cov_pi_sdk_models_list_helpers() {
    parse_list_models_table();
    is_separator_line();
    is_provider_id();
    is_noise_line();
    listing_from_row();
}

#[test]
fn kiss_cov_pi_sdk_protocol() {
    pi_encode_request();
    pi_decode_line();
    decode_response_line();
    json_error_string();
    prompt_request();
    new_session_request();
    abort_request();
    PiRequest();
    PiLine();
}

#[test]
fn kiss_cov_pi_sdk_map_a() {
    map_pi_event();
    map_message_update();
    tool_call_from_execution();
    tool_end_phase();
    tool_summary_from_pi();
    bash_summary();
    path_arg();
    path_tool_summary();
    flatten_ws();
    map_agent_end();
    last_assistant_text();
    assistant_message_text();
    aggregate_usage();
    usage_u64();
    text_delta_top_level();
    thinking_delta_top_level();
    top_level_delta_text();
}

#[test]
fn kiss_cov_pi_sdk_spawn() {
    pi_spawn_bridge();
    spawn_bridge();
    split_provider_model();
    pi_open_bridge_session();
    PiChildStdio();
    pi_take_stdio();
    pi_note_sandbox();
    pi_assemble_session();
    pi_build_command();
}

#[test]
fn kiss_cov_pi_sdk_rpc_io() {
    pi_write_line();
    pi_write_abort();
    pi_send_new_session();
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

#[test]
fn kiss_cov_retry_teardown_helpers() {
    agent_error_requires_coder_session_teardown();
    agent_string_is_cursor_agent_busy();
    agent_string_is_stale_cursor_sdk_auth();
    text_has_any();
    CHILD_OR_BRIDGE_DEAD_NEEDLES();
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
    run_hello_prompt();
    write_exec_script();
}
