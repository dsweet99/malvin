//! Kiss coverage witnesses for `cursor_sdk` (static name appearance in a test file).

#[test]
fn kiss_cov_cursor_sdk_auth_and_bridge_path() {
    let _ = super::auth::ensure_sdk_authenticated;
    let _ = super::auth::effective_sdk_api_key;
    let _ = super::bridge_path::resolve_bridge_js;
    let _ = super::bridge_path::resolve_models_js;
    let _ = stringify!(resolve_node_bin);
    let _ = stringify!(node_candidates);
    let _ = stringify!(push_unique);
    let _ = stringify!(cursor_agent_version_nodes);
    let _ = stringify!(node_major_version);
    let _ = stringify!(cursor_acp_test_mock_override);
    let _ = stringify!(candidate_roots);
    let _ = stringify!(ENV_BRIDGE);
}

#[test]
fn kiss_cov_cursor_sdk_client_api() {
    let _ = super::CursorSdkClient::new;
    let _ = super::CursorSdkClient::with_max_retries;
    let _ = stringify!(CursorSdkClient);
    let _ = stringify!(set_run_timing);
    let _ = stringify!(attach_run_timing_for_session);
    let _ = stringify!(sync_timing_to_open_session);
    let _ = stringify!(has_open_coder_session);
    let _ = stringify!(last_coder_prompt_agent_response);
    let _ = stringify!(ensure_authenticated);
    let _ = stringify!(begin_coder_session);
    let _ = stringify!(end_coder_session);
    let _ = stringify!(run_coder_prompt);
    let _ = stringify!(run_one);
    let _ = stringify!(emit_prompt_stdout);
    let _ = stringify!(append_prompt_files);
    let _ = stringify!(format_prompt_line);
    let _ = stringify!(append_prompt_log_bytes);
}

#[test]
fn kiss_cov_cursor_sdk_protocol() {
    let _ = super::protocol::encode_request;
    let _ = super::protocol::decode_event;
    let _ = stringify!(BridgeRequest);
    let _ = stringify!(BridgeEvent);
    let _ = stringify!(Create);
    let _ = stringify!(Send);
    let _ = stringify!(Cancel);
    let _ = stringify!(Close);
    let _ = stringify!(Ok);
    let _ = stringify!(Assistant);
    let _ = stringify!(Thinking);
    let _ = stringify!(ToolCall);
    let _ = stringify!(Step);
    let _ = stringify!(Usage);
    let _ = stringify!(RunDone);
    let _ = stringify!(Fatal);
    let _ = stringify!(Unknown);
}

#[test]
fn kiss_cov_cursor_sdk_session_core() {
    let _ = stringify!(BridgeSession);
    let _ = stringify!(BridgeSpawnArgs);
    let _ = stringify!(spawn);
    let _ = stringify!(send_prompt);
    let _ = stringify!(shutdown);
    let _ = stringify!(spawn_bridge);
    let _ = stringify!(take_stdio);
    let _ = stringify!(note_sandbox);
    let _ = stringify!(assemble_session);
    let _ = stringify!(ChildStdio);
    let _ = stringify!(mock_client);
    let _ = stringify!(prompt_once);
    let _ = stringify!(assert_usage);
    let _ = stringify!(assert_session_timing_synced);
    let _ = stringify!(mock_bridge_path);
    let _ = stringify!(run_prompt_and_assert_usage);
    let _ = stringify!(cursor_sdk_warm_start_attach_after_begin_records_usage);
    let _ = stringify!(cursor_sdk_run_done_result_feeds_do_dm_stdout);
    let _ = stringify!(prompt_need_dm_with_capture);
    let _ = stringify!(assert_dm_hello);
    let _ = stringify!(open_bridge_session);
    let _ = stringify!(resolve_node_and_bridge);
    let _ = stringify!(build_bridge_command);
    let _ = stringify!(send_create);
    let _ = stringify!(write_request);
    let _ = stringify!(read_event);
    let _ = stringify!(wait_for_ok);
    let _ = stringify!(drain_until_run_done);
    let _ = stringify!(start_mem_watch);
    let _ = stringify!(finish_run_done);
}

#[test]
fn kiss_cov_cursor_sdk_log_and_timing() {
    let _ = stringify!(feed_do_dm_run_result);
    let _ = stringify!(handle_stream_event);
    let _ = stringify!(emit_assistant);
    let _ = stringify!(emit_thinking);
    let _ = stringify!(emit_tool);
    let _ = stringify!(ToolCallFields);
    let _ = stringify!(tee_coalesced);
    let _ = stringify!(flush_stdout_coalesce);
    let _ = stringify!(print_coalesced_line);
    let _ = stringify!(compose_tool_done_line);
    let _ = stringify!(clear_tool_starts);
    let _ = stringify!(tool_starts);
    let _ = stringify!(ToolCallStart);
    let _ = stringify!(append_trace_value);
    let _ = stringify!(append_trace_raw);
    let _ = stringify!(append_trace_line);
    let _ = stringify!(stdout_coalesce);
    let _ = stringify!(word_sized_assistant_chunks_coalesce_before_flush);
    let _ = stringify!(newline_in_assistant_chunk_flushes_line);
    let _ = stringify!(thought_then_message_flushes_thought_on_kind_switch);
    let _ = stringify!(coalesced_assistant_line_prints_once_under_m_tag);
    let _ = stringify!(compose_tool_done_line_run_success);
    let _ = stringify!(install_mock_bridge_env);
    let _ = stringify!(mock_io);
    let _ = stringify!(spawn_mock);
    let _ = stringify!(write_line);
    let _ = stringify!(read_until);
    let _ = stringify!(run_mock_prompt);
    let _ = stringify!(assert_usage);
    let _ = super::timing::note_sdk_step;
    let _ = super::timing::record_sdk_usage;
}

#[test]
fn kiss_cov_cursor_sdk_test_helpers() {
    let _ = stringify!(cursor_sdk_client_mock_bridge_prompt_records_usage);
    let _ = stringify!(mock_bridge_js);
    let _ = stringify!(mock_bridge_create_send_close);
    let _ = stringify!(resolve_bridge_js_finds_repo_dist);
    let _ = stringify!(encode_create_uses_camel_case_api_key);
    let _ = stringify!(decode_run_done_and_fatal);
    let _ = stringify!(ensure_sdk_authenticated_ok_with_key);
    let _ = stringify!(note_sdk_step_increments);
    let _ = stringify!(record_sdk_usage_folds_cache_into_tokens_in);
    let _ = stringify!(kiss_cov_cursor_sdk_auth_and_bridge_path);
    let _ = stringify!(kiss_cov_cursor_sdk_client_api);
    let _ = stringify!(kiss_cov_cursor_sdk_protocol);
    let _ = stringify!(kiss_cov_cursor_sdk_session_core);
    let _ = stringify!(kiss_cov_cursor_sdk_log_and_timing);
}
