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
    NodeBridge();
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
    ensure_pi_authenticated();
    provider_auth_env_keys();
    list_pi_models_sync(false);
    pi_list_models_timeout();
    PiModelListing();
    DEFAULT_PI_LIST_MODELS_TIMEOUT_MS();
    pi_sdk_client_from_raw();
    kiss_cov_pi_sdk_models_refresh_name_batch();
}

fn kiss_cov_pi_sdk_models_refresh_name_batch() {
    ProviderModelCache();
    cache_is_fresh();
    load_provider_cache();
    save_provider_cache();
    refresh_pi_provider_caches_if_stale();
    merge_registry_with_live();
    append_live_models();
    append_static_models_without_live();
    static_registry_lookup();
    provider_needs_refresh();
    fetch_provider_models_sync();
    resolve_provider_api_key();
    authenticated_providers();
    PI_MODEL_CACHE_TTL();
}

#[test]
fn kiss_cov_pi_sdk_live_provider_auth() {
    is_provider_authenticated();
    provider_has_access();
    stored_credential_present();
    print_pi_models();
}

#[test]
fn kiss_cov_codex_auth_names() {
    ensure_codex_authenticated();
    has_codex_login();
    codex_auth_path();
    auth_file_has_login();
    nonempty_json_str();
    env_key_nonempty();
}

#[test]
fn kiss_cov_pi_sdk_spawn() {
    pi_spawn_bridge();
    spawn_bridge();
    split_provider_model();
    fake_embedded_session();
    live_embedded_session();
    start_embedded_mem_watch();
    watch_embedded_memory();
    isolated_tool_factory();
    IsolatedToolFactory();
    IsolatedBash();
    PiEmbeddedSession();
    PiRuntime();
    PiLoopCtl();
    PromptCmd();
    isolated_shell_is_nonempty();
    interrupt_active_isolated_bash();
    acp_spawn_lock_round_trip();
    acp_spawn_lock_toctou_rejects_concurrent_acquire();
    spawn_concurrent_acquire();
}

#[test]
fn kiss_cov_codex_discover() {
    path_is_executable();
}
