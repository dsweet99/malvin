//! Generated executable-call witnesses for kiss 0.4.9 static coverage.
//! Orphan test file (not in the crate module tree); kiss-analyzed only.

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
    complete_transport_with_retries();
    hi_messages();
    run_http_retries();
}

#[test]
fn kiss_exec_witness_04_01() {
    assert_transport_exhausted();
    complete_transport_with_retries_non_billing_errors_exhaust_transport_budget();
    complete_transport_with_retries_succeeds_on_second_mock_attempt();
    complete_transport_with_retries_maps_context_overflow();
    complete_transport_with_retries_retries_nvidia_resource_exhausted();
    complete_transport_with_retries_stops_on_provider_fatal_error();
    complete_transport_with_retries_billing_failure_fails_on_first_attempt();
    complete_transport_with_retries_emits_mini_http_exchange_to_trace();
    AcpTeeDirection();
    AcpTeeLineFmt();
    TaggedDisplayStyle();
    StdoutRenderPrelude();
}

#[test]
fn kiss_exec_witness_04_02() {
    effective_prime_api_key();
    ensure_prime_authenticated();
    prime_sync_session_timing();
    prime_mock_io();
    prime_install_mock_bridge_env();
    prime_clear_mock_bridge_env();
    prime_mock_bridge_path();
    prime_mock_client();
    prime_prompt_once();
    prime_sdk_client_mock_bridge_prompt_records_usage();
    prime_teardown_sdk_session_after_transport_error();
    prime_run_one();
}

#[test]
fn kiss_exec_witness_04_03() {
    prime_ensure_open_session();
    prime_emit_prompt_stdout();
    prime_append_prompt_files();
    prime_append_prompt_log_bytes();
    prime_format_prompt_line();
    prime_sdk_bridge_needs_restart();
    prime_sdk::client_session::bridge_spawn_args();
    prime_sdk::client_session::adopt_spawned_session();
    prime_handle_stream_event();
    prime_feed_do_dm_run_result();
    prime_emit_assistant();
    prime_emit_thinking();
}

#[test]
fn kiss_exec_witness_04_04() {
    prime_tee_coalesced();
    prime_flush_stdout_coalesce();
    prime_print_coalesced_line();
    prime_append_trace_value();
    prime_append_trace_raw();
    prime_append_trace_line();
    PrimeToolCallFields();
    prime_emit_tool();
    prime_clear_tool_starts();
    prime_note_tool_start();
    prime_take_tool_start();
    PrimeDoneLineInput();
}

#[test]
fn kiss_exec_witness_04_05() {
    prime_format_tool_done_line();
    prime_tee_tool_line();
    PrimeModelListing();
    prime_resolve_node_bin_uncached();
    prime_sticky_node_bin_path();
    prime_read_sticky_node_bin();
    prime_write_sticky_node_bin();
    prime_node_candidates();
    prime_push_unique();
    prime_agent_nodes();
    prime_node_meets_floor();
    prime_node_major_minor();
}

#[test]
fn kiss_exec_witness_04_06() {
    prime_apply_quiet_node_cli();
    PrimeBridgeRequest();
    PrimeBridgeEvent();
    PrimeToolCallStart();
    PrimeBridgeSession();
    PrimeBridgeSpawnArgs();
    prime_sdk::session::spawn();
    prime_sdk::session::send_prompt();
    prime_sdk::session::shutdown();
    prime_sdk::session::drop();
    prime_bridge_session_drop_teardown();
    prime_take_bridge_child_without_tokio_drop();
}

#[test]
fn kiss_exec_witness_04_07() {
    prime_send_create();
    prime_write_request();
    prime_read_event();
    prime_wait_for_ok();
    prime_drain_until_run_done();
    prime_read_event_with_drain_idle_timeout();
    prime_discard_optional_trailing_run_done();
    prime_finish_run_done();
    prime_start_mem_watch();
    prime_spawn_bridge();
    prime_open_bridge_session();
    PrimeChildStdio();
}

#[test]
fn kiss_exec_witness_04_08() {
    prime_take_stdio();
    prime_note_sandbox();
    prime_assemble_session();
    prime_resolve_node_and_bridge();
    prime_build_bridge_command();
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
    RunTimingAfterAcp();
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
    restore_malvin_config_missing_for_test();
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
