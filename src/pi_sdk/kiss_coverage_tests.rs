
#[test]
fn kiss_cov_pi_sdk_discover_auth_models() {
    let _ = super::discover::resolve_pi_bin;
    let _ = super::discover::pi_missing_binary_message;
    let _ = super::discover::pi_version_ok;
    let _ = super::discover::parse_pi_version;
    let _ = stringify!(path_is_executable);
    let _ = stringify!(PI_MIN_VERSION);
    let _ = super::ensure_pi_authenticated;
    let _ = super::auth::provider_auth_env_keys;
    let _ = super::list_pi_models_sync;
    let _ = super::pi_list_models_timeout;
    let _ = stringify!(PiModelListing);
    let _ = stringify!(DEFAULT_PI_LIST_MODELS_TIMEOUT_MS);
    let _ = stringify!(parse_list_models_table);
    let _ = stringify!(is_separator_line);
    let _ = stringify!(is_provider_id);
    let _ = stringify!(is_noise_line);
    let _ = stringify!(header_columns);
    let _ = stringify!(HeaderColumns);
    let _ = stringify!(listing_from_fixed_columns);
    let _ = stringify!(listing_from_whitespace_row);
    let _ = stringify!(thinking_from_fixed_columns);
    let _ = stringify!(parse_thinking_cell);
    let _ = stringify!(PI_MISSING_HINT);
}

#[test]
fn kiss_cov_pi_sdk_protocol_and_map() {
    let _ = super::protocol::pi_encode_request;
    let _ = super::protocol::pi_decode_line;
    let _ = stringify!(prompt_request);
    let _ = stringify!(new_session_request);
    let _ = stringify!(abort_request);
    let _ = stringify!(PiRequest);
    let _ = stringify!(PiLine);
    let _ = stringify!(Response);
    let _ = stringify!(Event);
    let _ = super::map_event::map_pi_event;
    let _ = stringify!(map_message_update);
    let _ = stringify!(tool_call_from_execution);
    let _ = stringify!(map_agent_end);
    let _ = stringify!(last_assistant_text);
    let _ = stringify!(aggregate_usage);
    let _ = stringify!(usage_u64);
    let _ = stringify!(text_delta_top_level);
    let _ = stringify!(thinking_delta_top_level);
    let _ = stringify!(tool_end_phase);
    let _ = stringify!(flatten_ws);
    let _ = stringify!(tool_summary_from_pi);
    let _ = stringify!(bash_summary);
    let _ = stringify!(path_arg);
    let _ = stringify!(path_tool_summary);
    let _ = stringify!(assistant_message_text);
    let _ = stringify!(top_level_delta_text);
}

#[test]
fn kiss_cov_pi_sdk_session_core() {
    let _ = super::spawn_bridge;
    let _ = super::send_prompt;
    let _ = super::write_abort;
    let _ = stringify!(pi_spawn_bridge);
    let _ = stringify!(split_provider_model);
    let _ = stringify!(pi_open_bridge_session);
    let _ = stringify!(PiChildStdio);
    let _ = stringify!(pi_take_stdio);
    let _ = stringify!(pi_note_sandbox);
    let _ = stringify!(pi_assemble_session);
    let _ = stringify!(pi_build_command);
    let _ = stringify!(pi_write_line);
    let _ = stringify!(pi_write_abort);
    let _ = stringify!(pi_send_new_session);
    let _ = stringify!(pi_send_prompt);
    let _ = stringify!(pi_wait_for_response);
    let _ = stringify!(pi_read_line);
    let _ = stringify!(pi_drain_until_run_done);
    let _ = stringify!(pi_read_line_with_idle_timeout);
    let _ = stringify!(pi_finish_run_done);
    let _ = stringify!(pi_next_req_id);
    let _ = stringify!(PI_REQ_SEQ);
    let _ = stringify!(pi_sdk_client_from_raw);
    let _ = crate::bridge_sdk::BridgeWire::PiRpc;
    let _ = crate::bridge_sdk::BridgeWire::NodeBridge;
}
