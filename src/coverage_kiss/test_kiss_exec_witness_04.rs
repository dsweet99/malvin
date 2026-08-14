
#[test]
fn kiss_exec_witness_04_00() {
    openrouter_error_maps_billing_failure();
    openrouter_mock_http_complete_returns_usage();
    openrouter_mock_http_complete_returns_usage_cost();
    openrouter_complete_transport_error_on_unreachable_host();
    openrouter_error_on_non_200_request_failed();
    openrouter_error_on_missing_content();
    HttpRetryLimits();
    HttpRetryCounters();
    record_outcome();
    hi_messages();
    run_http_retries();
}

#[test]
fn kiss_exec_witness_04_01() {
    assert_transport_exhausted();
    AcpTeeDirection();
    AcpTeeLineFmt();
    TaggedDisplayStyle();
    StdoutRenderPrelude();
}

#[test]
fn kiss_exec_witness_04_02() {
    cursor_sync_session_timing();
    cursor_mock_io();
    cursor_install_mock_bridge_env();
    cursor_clear_mock_bridge_env();
    cursor_mock_bridge_path();
    cursor_mock_client();
    cursor_prompt_once();
    cursor_teardown_sdk_session_after_transport_error();
    cursor_run_one();
}

#[test]
fn kiss_exec_witness_04_03() {
    cursor_ensure_open_session();
    cursor_emit_prompt_stdout();
    cursor_append_prompt_files();
    cursor_append_prompt_log_bytes();
    cursor_format_prompt_line();
    cursor_sdk_bridge_needs_restart();
    cursor_sdk::client_session::bridge_spawn_args();
    cursor_sdk::client_session::adopt_spawned_session();
    cursor_handle_stream_event();
    cursor_feed_do_dm_run_result();
    cursor_emit_assistant();
    cursor_emit_thinking();
}

#[test]
fn kiss_exec_witness_04_04() {
    cursor_tee_coalesced();
    cursor_flush_stdout_coalesce();
    cursor_print_coalesced_line();
    cursor_append_trace_value();
    cursor_append_trace_raw();
    cursor_append_trace_line();
    ToolCallFields();
    cursor_emit_tool();
    cursor_clear_tool_starts();
    cursor_note_tool_start();
    cursor_take_tool_start();
    DoneLineInput();
}

#[test]
fn kiss_exec_witness_04_05() {
    cursor_format_tool_done_line();
    cursor_tee_tool_line();
    cursor_resolve_node_bin_uncached();
    cursor_sticky_node_bin_path();
    cursor_read_sticky_node_bin();
    cursor_write_sticky_node_bin();
    cursor_node_candidates();
    cursor_push_unique();
    cursor_agent_nodes();
    cursor_node_meets_floor();
    cursor_node_major_minor();
}

#[test]
fn kiss_exec_witness_04_06() {
    cursor_apply_quiet_node_cli();
    BridgeRequest();
    BridgeEvent();
    ToolCallStart();
    BridgeSession();
    BridgeSpawnArgs();
    cursor_sdk::session::spawn();
    cursor_sdk::session::send_prompt();
    cursor_sdk::session::shutdown();
    cursor_sdk::session::drop();
    bridge_session_drop_teardown();
    take_bridge_child_without_tokio_drop();
}

#[test]
fn kiss_exec_witness_04_07() {
    cursor_send_create();
    cursor_write_request();
    cursor_read_event();
    cursor_wait_for_ok();
    cursor_drain_until_run_done();
    cursor_read_event_with_idle_timeout();
    cursor_discard_optional_trailing_run_done();
    cursor_finish_run_done();
    cursor_start_mem_watch();
    cursor_spawn_bridge();
    cursor_open_bridge_session();
    CursorChildStdio();
    create_ack_idle_timeout_fails_begin();
    empty_result_run_done_clears_prior_last_response();
}

#[test]
fn kiss_exec_witness_04_08() {
    cursor_take_stdio();
    cursor_note_sandbox();
    cursor_assemble_session();
    cursor_resolve_node_and_bridge();
    cursor_build_bridge_command();
    scrub_cursor_keys();
    apply_node_compile_cache();
    EnvHomeGuard();
    prompts::embedded_defaults_tests::drop();
    KpopPromptValidation();
    prompt_source_desc();
    ReliabilityTierFlags();
}

#[test]
fn kiss_exec_witness_04_09() {
    RunTimingSessionEnd();
    RunTimingAfterBackend();
    TimingPhase();
    AcpStepProxy();
    CostPolicy();
    format_token_field();
    format_cost_field();
    token_fields_fragment();
    cost_fields_fragment();
    SandboxOomKillFacts();
    DotfileBackupLabels();
}

#[test]
fn kiss_exec_witness_04_10() {
    random_backup_id();
    GitignoreFileBackup();
    GitignoreBackup();
    seed_nested_gitignore_repo();
    tamper_gitignore_tree();
    assert_gitignore_contents();
    DotfileBackupPayload();
    SessionDotfileParts();
    DotfileSpecRow();
    labels_for_test();
    dotfile_spec_row_field_count();
    write_merged_default_malvin_config();
}

#[test]
fn kiss_exec_witness_04_11() {
    VisionFileBackup();
    VisionBackup();
    seed_nested_vision_repo();
    tamper_vision_tree();
    assert_vision_contents();
    NameFileState();
    generate_auto_name();
    release_name();
    assert_no_peer_name_lock();
    SessionNameGuard();
    session_name::drop();
    TerminalTheme();
}

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
