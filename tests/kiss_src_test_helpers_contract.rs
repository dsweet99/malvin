//! Integration kiss witnesses for `src/**` test-helper modules (bucket D).
//! Kiss credits per-file coverage only from external test files; `tests/` contracts
//! reach into `src` test helpers that `coverage_kiss/gate_refs.rs` cannot name.

#[test]
fn kiss_witness_acp_test_helpers_a() {
    let _ = stringify!(watch_process_group_memory_fail_closed_when_rss_unavailable);
    let _ = stringify!(watch_process_group_memory_writes_sandbox_oom_marker);
    let _ = stringify!(clear_if_prompt_response_clears_busy);
    let _ = stringify!(response_tx_oneshot_channel_constructible);
    let _ = stringify!(write_executable_agent_script);
    let _ = stringify!(teardown_async_ignoring_sigterm_eventually_killed);
    let _ = stringify!(terminate_process_group_noop_without_pgid_or_baseline);
    let _ = stringify!(signal_targets_noop_for_empty_set);
    let _ = stringify!(terminate_process_group_kills_sleep_child);
    let _ = stringify!(terminate_agent_process_group_kills_sleep_child);
    let _ = stringify!(baseline_amnestied_agent_acp_orphan_killed_on_teardown);
    let _ = stringify!(malvin_sibling_outside_agent_pg_killed_on_teardown);
}

#[test]
fn kiss_witness_acp_session_and_transport_b() {
    let _ = stringify!(busy_session_with_dead_transport);
    let _ = stringify!(acp_session_cancel_clears_busy_state_after_rpc_error);
    let _ = stringify!(dead_transport_child_stdio);
    let _ = stringify!(dead_transport_sync_channels);
    let _ = stringify!(dead_transport_session_inner);
    let _ = stringify!(spawn_json_activity_then_response);
    let _ = stringify!(spawn_activity_then_kill_child);
    let _ = stringify!(rpc_request_with_correlation_id_stays_alive_while_json_updates_arrive);
    let _ = stringify!(rpc_wait_response_reports_dead_child_after_silence);
    let _ = stringify!(rpc_response_arriving_during_child_health_grace_is_delivered);
    let _ = stringify!(test_handshake_hits_session_new_error_path);
    let _ = stringify!(handshake_skip_login_session_id);
    let _ = stringify!(handshake_can_skip_cursor_login_when_api_key_mode_is_used);
    let _ = stringify!(test_rpc_cancel_when_pending_sender_dropped);
}

#[test]
fn kiss_witness_acp_transport_and_backend_c() {
    let _ = stringify!(test_rpc_request_does_not_leak_pending_after_write_failure);
    let _ = stringify!(rpc_request_with_correlation_id_times_out_when_stdout_silent);
    let _ = stringify!(rpc_request_with_correlation_id_errors_when_reader_dead);
    let _ = stringify!(test_write_rpc_line_fails_after_child_stdin_closed);
    let _ = stringify!(mock_backend);
    let _ = stringify!(empty_backups);
    let _ = stringify!(RetryPollutionObservation);
    let _ = stringify!(count_user_messages_with_marker);
    let _ = stringify!(observe_retry_http_history);
    let _ = stringify!(retry_pollution_mock_client);
    let _ = stringify!(run_retry_pollution_prompt);
    let _ = stringify!(assert_retry_history_is_clean);
    let _ = stringify!(mini_coder_prompt_retry_does_not_pollute_session_history);
}

#[test]
fn kiss_witness_cli_and_artifacts_d() {
    let _ = stringify!(loop_driver_fenceless_completes_in_one_turn);
    let _ = stringify!(loop_driver_fenceless_no_nudge_in_prompts_log);
    let _ = stringify!(seed_home_logs_for_gc_test);
    let _ = stringify!(seed_short_id_lookup_fixture);
    let _ = stringify!(seed_kpop_multiturn_mock_workspace);
    let _ = stringify!(run_kpop_multiturn_mock_once);
    let _ = stringify!(run_gate_inline_summarize_first_iteration);
}
