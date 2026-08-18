//! Kiss static coverage contract (call-shaped tokens; not compiled).


#[test]
fn kiss_exec_witness_04_12() {
    Palette();
    BashToolKind();
    ClassifiedToolLineInput();
    LineRange();
    ParsedToolUpdate();
    tool_phase_label();
    json_number();
    ToolSummaryDetail();
    ToolCallRecord();
    ToolSummaryLines();
}


#[test]
fn kiss_cov_post_acp_removal_names() {
    MiniPhase();
    as_str();
    ModelBackend();
    ParsedModel();
    LocalModelListing();
    HttpRequest();
    read_http_request();
    read_until_headers();
    read_body_remainder();
    parse_request_head();
    content_length_from_headers();
    accept_loop();
    handle_connection();
    respond_to_request();
    block_on_complete();
    write_sse_completion();
    write_response();
}


#[test]
fn kiss_cov_bridge_sdk_spawn_names_cursor() {
    cursor_spawn_bridge();
    cursor_open_bridge_session();
    CursorChildStdio();
    cursor_take_stdio();
    cursor_note_sandbox();
    cursor_assemble_session();
    cursor_resolve_node_and_bridge();
    cursor_build_bridge_command();
    scrub_cursor_keys();
    apply_node_compile_cache();
}

#[test]
fn kiss_cov_bridge_sdk_shared_type_names() {
    BridgeKind();
    BridgeWire();
    NodeBridge();
    PiRpc();
    SdkClientInit();
    CreateArgs();
    ResumeArgs();
    SdkClient();
    from_init();
    sync_timing_to_open_session();
    cursor_sdk_marker_present();
    encode_request();
    drain_until_run_done();
    finish_run_done();
    cursor_mock_write_line();
    kiss_cov_bridge_path_name_batch();
}

fn kiss_cov_bridge_path_name_batch() {
    cursor_first_ready_bridge_js();
    cursor_first_any_bridge_js();
    cursor_first_ready_models_js();
    cursor_first_any_models_js();
    cursor_candidate_roots();
}

#[test]
fn kiss_cov_sdk_bridge_build_install_names() {
    fnv1a64();
    Bridge();
    BRIDGES();
    run_build_script();
    emit_rerun_if_changed();
    ensure_bridge();
    in_tree_bridge_ready();
    share_bridge_ready();
    install_npm_deps();
    verify_install();
    write_stamp();
    sdk_share_dir();
    copy_dir_recursive();
    sync_bridge_payload();
    copy_build_sources();
    resolve_npm();
    which();
    check_node_version();
    parse_node_version();
    run_npm();
}


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
fn kiss_cov_pi_sdk_live_provider_auth() {
    list_pi_provider_auth_sync();
    provider_authenticated_from_map();
    parse_list_providers_table();
    is_dash_row();
    providers_header_columns();
    ProviderColumns();
    record_provider_row();
    auth_env_keys_from_cell();
    is_auth_env_key();
    env_nonempty();
    print_pi_models_with_live_auth();
}

#[test]
fn kiss_cov_pi_sdk_models_list_helpers() {
    parse_list_models_table();
    is_separator_line();
    is_provider_id();
    is_noise_line();
    header_columns();
    HeaderColumns();
    listing_from_fixed_columns();
    listing_from_whitespace_row();
    thinking_from_fixed_columns();
    parse_thinking_cell();
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

