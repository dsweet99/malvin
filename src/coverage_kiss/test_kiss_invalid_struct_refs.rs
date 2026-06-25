//! External kiss struct value refs (witness hub; kiss-analyzed standalone).

#[test]
fn kiss_invalid_struct_value_refs_0() {
    let _ = crate::acp::client_impl_helpers::teardown_coder_session_after_transport_error;
    let _ = crate::acp::client_impl_prompt_dispatch::CoderSessionPromptDispatch;
    let _ = crate::acp::client_impl_prompt_retry::run_coder_prompt_with_retries;
    let _ = crate::acp::client_impl_prompt_retry::run_one_coder_prompt_attempt;
    let _ = crate::acp::AcpHandshakeIo;
    let _ = crate::acp::AcpChildStdout;
    let _ = crate::acp::ops_body_kpop::KpopPromptRound;
    let _ = crate::acp::ops_body_kpop::AgentKpopMultiturnCtl;
    let _ = crate::acp::ops_body_kpop::KpopFailAfterPrompt;
    let _ = crate::acp::ops_body_kpop_mt::MultiturnRoundAfter;
    let _ = crate::acp::prompt_trace_writer::LivePromptTraceArgs;
    let _ = crate::acp::prompt_trace_writer::register_deferred_sink;
}

#[test]
fn kiss_invalid_struct_value_refs_1() {
    let _ = crate::acp::PromptRpcCleanup;
    let _ = crate::acp::ReaderSpawnArgs;
    let _ = crate::acp::ReaderLoopInput;
    let _ = crate::acp::IncomingLineDispatch;
    let _ = crate::acp::ReaderLoopFinishCtx;
    let _ = crate::acp::ReaderLoopLineIo;
    let _ = crate::acp::ReaderLoopDrainCtx;
    let _ = crate::acp_tests::reader_tests_helpers::acp_activity_state;
    let _ = crate::acp_tests::reader_tests_helpers::dispatch_parts;
    let _ = crate::acp_tests::reader_tests_helpers::finish_stdout;
    let _ = crate::acp_tests::reader_tests_reader_loop::EofReaderSpawnInputs;
    let _ = crate::acp_tests::reader_tests_trace_kpop_helpers::KpopStdoutTraceFixture;
}

#[test]
fn kiss_invalid_struct_value_refs_2() {
    let _ = crate::acp::acp_activity_state;
    let _ = crate::acp::SessionInnerAssembly;
    let _ = crate::acp::SessionAfterStdioIn;
    let _ = crate::acp::stdin_from_sleep_holder;
    let _ = crate::acp::session_channel_state_sets_trace_jsonl_when_prompts_log_run_dir_set;
    let _ = crate::acp::session_types::AcpSessionInner;
    let _ = crate::acp::session_types::AcpSpawnArgs;
    let _ = crate::acp::trace_line_write::ReaderTraceLineOpts;
    let _ = crate::acp::trace_line_write::WriteTraceLineCoalescedOpts;
    let _ = crate::acp::trace_line_write::TraceFileStdout;
    let _ = crate::acp::trace_line_write_tool_summary::TeeToolSummaryPlainCtx;
    let _ = crate::acp::handshake::HandshakeParams;
}

#[test]
fn kiss_invalid_struct_value_refs_3() {
    let _ = crate::acp::rpc_part1::AcpStdioRpc;
    let _ = crate::acp::rpc_part1::RpcLineWriteOpts;
    let _ = crate::acp::rpc_part1::RpcOutgoing;
    let _ = crate::acp::rpc_part1::RpcRequestNext;
    let _ = crate::acp::rpc_wait_args::RpcWaitArgs;
    let _ = crate::acp::unix_process_group_teardown_poll::unix_process_group_teardown_timing::shutdown_cancel_timeout;
    let _ = crate::acp_transport_tests::shared_handshake::TestReaderLoopSpawn;
    let _ = crate::acp_transport_tests::shared_handshake::HandshakeRunning;
    let _ = crate::acp_transport_tests::shared_harness::InactiveRpcIo;
    let _ = crate::acp_transport_tests::shared_harness::SleepStdoutDrainMode;
    let _ = crate::acp_transport_tests::shared_harness::HarnessRpcWaitParams;
    let _ = crate::agent_backend::mini::client_prompt_log::PromptLogWrite;
}

#[test]
fn kiss_invalid_struct_value_refs_4() {
    let _ = crate::agent_backend::mini::client_retry_tests::RetryPollutionObservation;
    let _ = crate::agent_backend::mini::loop_driver::loop_http::HttpRetryRequest;
    let _ = crate::agent_backend::mini::loop_driver::loop_inner::CompleteTurnRequest;
    let _ = crate::agent_backend::mini::loop_driver::loop_types::LoopDriverRun;
    let _ = crate::child_health::evaluate_after_acp_silence;
    let _ = crate::cli::bug_id_lookup::bug_id_lookup_log::log_tag;
    let _ = crate::cli::bug_id_lookup::bug_id_lookup_log::missing_log_err_label;
    let _ = crate::cli::bug_id_lookup::bug_id_lookup_log::fallback_err_label;
    let _ = crate::cli::code_flow::run_loop::CodeGateFinish;
    let _ = crate::cli::do_flow::DoRunPrep;
    let _ = crate::cli::explain_flow::run_startup::ExplainKpopPrepared;
    let _ = crate::cli::kpop_flow::kpop_flow_a::KpopPrepared;
}

#[test]
fn kiss_invalid_struct_value_refs_5() {
    let _ = crate::cli::kpop_flow::kpop_flow_a::KpopArtifactsEarly;
    let _ = crate::cli::kpop_flow::kpop_flow_a::KpopAcpMultiturnCtx;
    let _ = crate::cli::kpop_flow::kpop_flow_run_loop::RunKpopAgentLoopsParams;
    let _ = crate::cli::kpop_flow::kpop_flow_run_loop::RunKpopAgentLoopsOutcome;
    let _ = crate::cli::kpop_flow::kpop_flow_run_loop::KpopLoopSnapshot;
    let _ = crate::cli::kpop_flow::kpop_flow_run_loop::KpopLoopExitAfterIteration;
    let _ = crate::cli::kpop_summarize::kpop_summarize_inline::InlineSummarizeOnKpopLoopCtx;
    let _ = crate::cli::kpop_summarize::kpop_summarize_inline::GateInlineSummarizeCtx;
    let _ = crate::cli::workflow_kpop_shared::workflow_kpop_render::RenderKpopProgram;
    let _ = crate::kpop_multiturn_prompts::SmokeKpopBuilder;
    let _ = crate::kpop_test_stubs::EchoPrompts;
    let _ = crate::kpop_test_stubs::kpop_block;
}

#[test]
fn kiss_invalid_struct_value_refs_6() {
    let _ = crate::prompts::embedded_defaults_tests::EnvHomeGuard;
    let _ = crate::prompts::embedded_defaults_tests::drop;
    let _ = crate::acp::VerboseTraceCoalesceState;
    let _ = crate::acp::MemWatchHandles;
    let _ = crate::active_agent_heartbeat::ActiveAgentSandbox;
    let _ = crate::active_agent_heartbeat::ActiveAgentStatsSource;
    let _ = crate::cli::bug_id_lookup::BugLogMatch;
    let _ = crate::cli::bug_id_lookup::BugIdResolved;
    let _ = crate::cli::explain_flow::run_loop::ExplainFinishInput;
}

#[test]
fn kiss_invalid_struct_value_refs_gate10_0() {
    let _ = crate::acp::process_group_mem_watch_tests::watch_process_group_memory_fail_closed_when_rss_unavailable;
    let _ = crate::acp::process_group_mem_watch_tests::watch_process_group_memory_writes_sandbox_oom_marker;
    let _ = crate::acp::reader_inline_tests::clear_if_prompt_response_clears_busy;
    let _ = crate::acp::session_drop_teardown::take_child_without_tokio_drop;
    let _ = crate::acp::session_drop_teardown::acp_session_drop_teardown;
    let _ = crate::acp::session_drop_teardown::take_child_without_tokio_drop_for_test;
    let _ = crate::acp::session_drop_teardown::acp_session_drop_if_last;
    let _ = crate::acp::session_types_tests::response_tx_oneshot_channel_constructible;
    let _ = crate::acp::trace_line_write_tee_tests::rendered_tool_summary_tee_display;
    let _ = crate::acp::transport::command_kiss_cov_tests::write_executable_agent_script;
    let _ = crate::acp::transport::rpc_part2::spawn_cat_rpc_stdio_pair;
    let _ = crate::acp::transport::rpc_part2::read_first_stdout_line;
}

#[test]
fn kiss_invalid_struct_value_refs_gate10_1() {
    let _ = crate::acp::transport::rpc_part2::write_rpc_line_appends_flush_line_readable_on_child_stdout;
    let _ = crate::acp::unix_process_group_teardown::signal_targets;
    let _ = crate::acp::unix_process_group_teardown_escalation_tests::teardown_async_ignoring_sigterm_eventually_killed;
    let _ = crate::acp::unix_process_group_teardown_escalation_tests::terminate_process_group_noop_without_pgid_or_baseline;
    let _ = crate::acp::unix_process_group_teardown_tests::signal_targets_noop_for_empty_set;
    let _ = crate::acp::unix_process_group_teardown_tests::terminate_process_group_kills_sleep_child;
    let _ = crate::acp::unix_process_group_teardown_tests::terminate_agent_process_group_kills_sleep_child;
    let _ = crate::acp::unix_process_group_teardown_tests::baseline_amnestied_agent_acp_orphan_killed_on_teardown;
    let _ = crate::acp::unix_process_group_teardown_tests::malvin_sibling_outside_agent_pg_killed_on_teardown;
    let _ = crate::acp_session_tests::cancel::busy_session_with_dead_transport;
    let _ = crate::acp_session_tests::cancel::acp_session_cancel_clears_busy_state_after_rpc_error;
    let _ = crate::acp_session_tests::session_inner::dead_transport_child_stdio;
}

#[test]
fn kiss_invalid_struct_value_refs_gate10_2() {
    let _ = crate::acp_session_tests::session_inner::dead_transport_sync_channels;
    let _ = crate::acp_session_tests::session_inner::dead_transport_session_inner;
    let _ = crate::acp_session_tests::unix_helpers::wait_for_pid_file;
    let _ = crate::acp_session_tests::unix_helpers::write_descendant_spawning_acp_mock;
    let _ = crate::acp_session_tests::unix_shutdown::shutdown_sends_cancel_before_teardown;
    let _ = crate::acp_session_tests::unix_shutdown::shutdown_kills_agent_spawned_descendants;
    let _ = crate::acp_test_mock_js::acp_mock_js;
    let _ = crate::acp_transport_tests::child_health_a::spawn_json_activity_then_response;
    let _ = crate::acp_transport_tests::child_health_a::spawn_activity_then_kill_child;
    let _ = crate::acp_transport_tests::child_health_a::rpc_request_with_correlation_id_stays_alive_while_json_updates_arrive;
    let _ = crate::acp_transport_tests::child_health_a::rpc_wait_response_reports_dead_child_after_silence;
    let _ = crate::acp_transport_tests::child_health_b::rpc_response_arriving_during_child_health_grace_is_delivered;
}

#[test]
fn kiss_invalid_struct_value_refs_gate10_3() {
    let _ = crate::acp_transport_tests::jsonrpc::command_env_value;
    let _ = crate::acp_transport_tests::rpc_integration_a1::test_handshake_hits_session_new_error_path;
    let _ = crate::acp_transport_tests::rpc_integration_a2::handshake_skip_login_session_id;
    let _ = crate::acp_transport_tests::rpc_integration_a2::handshake_can_skip_cursor_login_when_api_key_mode_is_used;
    let _ = crate::acp_transport_tests::rpc_integration_a2::test_rpc_cancel_when_pending_sender_dropped;
    let _ = crate::acp_transport_tests::rpc_integration_b::test_rpc_request_does_not_leak_pending_after_write_failure;
    let _ = crate::acp_transport_tests::rpc_integration_b::rpc_request_with_correlation_id_times_out_when_stdout_silent;
    let _ = crate::acp_transport_tests::rpc_integration_b::rpc_request_with_correlation_id_errors_when_reader_dead;
    let _ = crate::acp_transport_tests::rpc_unit::test_write_rpc_line_fails_after_child_stdin_closed;
    let _ = crate::acp_transport_tests::shared_handshake::write_bad_session_new_mock;
    let _ = crate::acp_transport_tests::shared_handshake::write_authenticate_rejected_but_session_new_ok_mock;
    let _ = crate::agent_backend::backend_kpop_tests::mock_backend;
}

#[test]
fn kiss_invalid_struct_value_refs_gate10_4() {
    let _ = crate::agent_backend::backend_kpop_tests::empty_backups;
    let _ = crate::agent_backend::mini::client_retry_tests::count_user_messages_with_marker;
    let _ = crate::agent_backend::mini::client_retry_tests::observe_retry_http_history;
    let _ = crate::agent_backend::mini::client_retry_tests::retry_pollution_mock_client;
    let _ = crate::agent_backend::mini::client_retry_tests::run_retry_pollution_prompt;
    let _ = crate::agent_backend::mini::client_retry_tests::assert_retry_history_is_clean;
    let _ = crate::agent_backend::mini::client_retry_tests::mini_coder_prompt_retry_does_not_pollute_session_history;
    let _ = crate::agent_backend::mini::loop_driver_no_fence_tests::loop_driver_no_fence_triggers_nudge_before_final;
    let _ = crate::agent_backend::mini::loop_driver_no_fence_tests::loop_driver_fenceless_after_nudge_without_bash_errors;
    let _ = crate::agent_backend::mini::loop_driver_tests::loop_driver_single_fence_runs_bash_and_appends_observation;
    let _ = crate::agent_backend::mini::loop_driver_tests::loop_driver_mini_done_line_terminates;
    let _ = crate::agent_backend::mini::loop_driver_tests::loop_driver_mini_done_inside_fence_still_runs_bash;
}

#[test]
fn kiss_invalid_struct_value_refs_gate10_5() {
    let _ = crate::agent_backend::mini::loop_driver_tests::loop_driver_prepends_mini_constraints;
    let _ = crate::agent_backend::mini::loop_driver_tests::loop_driver_mock_http_retry_on_429;
    let _ = crate::agent_backend::mini::trace_tests::stdout_log_tool_t_lines;
    let _ = crate::artifacts::log_gc_hook_tests::seed_home_logs_for_gc_test;
    let _ = crate::cli::config_defaults_tests::assert_workflow_defaults;
    let _ = crate::cli::kpop_flow_a_tests::seed_short_id_lookup_fixture;
    let _ = crate::cli::kpop_flow_a_tests::seed_kpop_multiturn_mock_workspace;
    let _ = crate::cli::kpop_flow_a_tests::run_kpop_multiturn_mock_once;
    let _ = crate::cli::kpop_summarize_inline_tests::run_gate_inline_summarize_first_iteration;
    let _ = crate::cli::kpop_summarize_mock_tests::write_summarize_fixture_exp_logs;
    let _ = crate::cli::kpop_summarize_mock_tests::run_inline_summarize_on_open_mock_session;
    let _ = crate::cli::repo_checks::tests_gates_unix::test_scan_for_extension_handles_symlink_cycles;
}

#[test]
fn kiss_invalid_struct_value_refs_gate10_6() {
    let _ = crate::repo_gates::discover_init_checks_fixtures::write_repo_files;
}
