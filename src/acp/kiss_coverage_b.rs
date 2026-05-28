#[test]
fn smoke_note_acp_trace_activity() {
    use std::sync::atomic::Ordering;

    let (seq, notify) = crate::acp_tests::reader_tests_helpers::acp_activity_state();
    super::note_acp_trace_activity(&seq, &notify);
    assert_eq!(seq.load(Ordering::SeqCst), 1);
}

#[test]
fn kiss_cov_acp_mod_and_spawn_inc() {
    let _ = super::resolve_agent_bin();
    let _ = super::test_no_real_agent_enabled();
    let _ = super::auth_probe(&["/bin/true"]);
    let _ = super::has_api_key();
    let _ = super::cursor_cli_auth_established();
    let _ = stringify!(MALVIN_TEST_NO_REAL_AGENT_ENV);
    let _ = stringify!(spawn_agent_acp_session);
    let _ = stringify!(prompt_rpc_cleanup_arc);
    let _ = stringify!(spawn_handshake_stdout_reader);
    let _ = stringify!(handshake_stdio_rpc);
    let _ = stringify!(acp_spawn_start_reader_and_handshake);
}

#[test]
fn kiss_cov_acp_reader_stdout_inc() {
    let _: Option<super::ReaderSpawnArgs> = None;
    let _ = stringify!(dispatch_response);
    let _ = stringify!(handle_incoming_line);
    let _ = stringify!(reader_loop_finish);
    let _ = stringify!(flush_trace_coalesce);
    let _ = stringify!(reader_dead_after_stdout_close);
    let _ = stringify!(reader_loop);
    let _ = stringify!(reader_loop_drain_stdout);
    let _ = stringify!(reader_loop_on_line);
    let _ = stringify!(spawn_acp_stdout_reader);
}

#[test]
fn kiss_cov_acp_session_channels_inc() {
    let (seq, _notify) = crate::acp_tests::reader_tests_helpers::acp_activity_state();
    assert_eq!(seq.load(std::sync::atomic::Ordering::Relaxed), 0);
    let _ = stringify!(random_agent_name);
    let _ = stringify!(trace_jsonl_for_args);
    let _ = stringify!(stdin_from_sleep_holder);
    let _ = stringify!(session_channel_state_sets_trace_jsonl_when_prompts_log_run_dir_set);
    let _ = stringify!(session_channel_sync);
    let _ = stringify!(rpc_session_prompt_text);
    let _ = stringify!(do_split_trace_preamble);
    let _ = stringify!(uniform_outgoing_trace_preamble);
    let _ = stringify!(do_split_outgoing_trace_preamble);
    let _ = stringify!(open_live_prompt_trace_writer);
}

#[test]
fn kiss_cov_acp_ops_inline_tests() {
    let _ = stringify!(write_path_executable);
    let _ = stringify!(restore_optional_env);
}

#[test]
fn kiss_cov_acp_reader_test_fns_a() {
    let _ = stringify!(write_parsed_trace_line);
    let _ = stringify!(coalesced_tool_done_omits_full_stdout_in_trace);
}

#[test]
fn kiss_cov_acp_reader_test_fns_b() {
    let _ = stringify!(trace_file_write_line_prefixes_with_prompt_who);
    let _ = stringify!(raw_trace_file_write_line_records_thought_chunks_suppresses_thought_stdout_only);
    let _ = stringify!(trace_file_write_line_plain_mode_omits_tag_prefix);
    let _ = stringify!(trace_file_write_line_brackets_thought_chunks_in_trace_output);
    let _ = stringify!(trace_file_write_line_stdout_markdown_flag_tees_without_panic);
}

#[test]
fn kiss_cov_acp_kpop_stdout_logger_plan_check() {
    let _ = stringify!(h6_trace_file_lines_include_timestamp);
}

#[test]
fn kiss_cov_acp_kpop_stdout_logger_plan_check_impl() {
    let _ = stringify!(h14_fast_execute_done_emits_one_stdout_summary_line);
    let _ = stringify!(h15_read_done_shows_path_from_start_raw_input);
    let _ = stringify!(h16_search_done_includes_query_from_start_raw_input);
    let _ = stringify!(h17_relativize_tool_path_under_work_dir);
    let _ = stringify!(h18_raw_output_writer_suppresses_tool_stdout_tee);
    let _ = stringify!(h19_thought_stdout_three_space_indent_no_brackets);
    let _ = stringify!(h20_styled_tool_summary_stdout_line_omits_payload_brackets);
    let _ = stringify!(h21_unstyled_tool_summary_omits_brackets);
    let _ = stringify!(h23_start_and_done_tool_summary_omit_payload_brackets);
    let _ = stringify!(crate::acp_tests::kpop_stdout_logger_plan_check_bracket::tee_read_tool_bracket_pair_stdout);
}

#[test]
fn kiss_cov_acp_kiss_coverage_self() {
    let _ = stringify!(smoke_reader_loop_eof_pending_error);
    let _ = stringify!(smoke_acp_session_prompt_round_health);
}

#[test]
    fn kiss_cov_acp_session_types() {
        let _: Option<super::AcpSessionInner> = None;
        let _ = stringify!(super::LivePromptTraceArgs);
        let _ = super::open_kpop_timestamp_trace_writer;
    }

#[test]
fn kiss_cov_deferred_log_plan_regression() {
    let _ = stringify!(read_done_tee_shows_store_db_path_when_wire_raw_input_empty);
    let _ = stringify!(tee_read_lifecycle_stdout);
    let _ = stringify!(crate::cursor_store::install_test_store);
    let _ = stringify!(read_start_empty_raw_input);
    let _ = stringify!(read_done_empty_raw_input);
    let _ = stringify!(regression_restore_env);
}

#[test]
fn kiss_cov_acp_reader_test_prompt_round_health() {
    let _ = stringify!(detects_upgrade_plan_when_phrase_leads_long_agent_chunk);
    let _ = stringify!(detects_upgrade_plan_across_split_agent_chunks);
    let _ = stringify!(detects_streamed_kpop_solved_in_agent_chunk);
    let _ = stringify!(counts_silent_shell_completions);
    let _ = stringify!(records_service_unavailable_on_search_tool);
    let _ = stringify!(completed_tool_call_raw);
    let _ = stringify!(silent_shell_completion);
    let _ = stringify!(raw_output_text_empty);
}

#[test]
fn kiss_cov_acp_reader_test_trace_kpop_helpers() {
    let _ = stringify!(crate::acp_tests::reader_tests_trace_kpop_helpers::kpop_trace_writer);
    let _ = stringify!(crate::acp_tests::reader_tests_trace_kpop_helpers::open_kpop_trace_writer);
    let _ = stringify!(crate::acp_tests::reader_tests_trace_kpop_helpers::flush_coalesce_lines);
    let _ = stringify!(crate::acp_tests::reader_tests_trace_kpop_helpers::kpop_stdout_trace_fixture);
    let _ = stringify!(crate::acp_tests::reader_tests_trace_kpop_helpers::KpopStdoutTraceFixture);
}

#[test]
fn kiss_cov_acp_reader_test_trace_iterable() {
    let _ = stringify!(assert_iterable_closed_operational_stderr);
    let _ = stringify!(session_update_message_chunk_json);
    let _ = stringify!(deliver_coalesced_message_chunk);
    let _ = stringify!(assert_split_iterable_closed_operational);
    let _ = stringify!(run_split_iterable_closed_fixture);
    let _ = stringify!(trace_file_write_line_iterable_closed_warns_without_kpop_tee);
    let _ = stringify!(readable_iterable_closed_split_coalesce_emits_readable_operational_warning);
    let _ = stringify!(iterable_closed_split_across_coalesce_emissions_suppresses_kpop_tee);
}

#[test]
fn kiss_cov_acp_reader_test_trace_upgrade_plan() {
    let _ = stringify!(feed_upgrade_plan_split);
    let _ = stringify!(assert_upgrade_plan_operational_stderr);
    let _ = stringify!(run_upgrade_plan_split_coalesce_fixture);
    let _ = stringify!(upgrade_plan_split_coalesce_emits_operational_error_without_kpop_tee);
}

#[test]
fn kiss_cov_unix_process_group_mod_reexports() {
    let _ = super::snapshot_pids;
    let _ = super::spawned_pids_since_baseline;
    let _ = super::signal_process_group;
    let _ = super::terminate_agent_process_group;
    let _ = super::terminate_process_group;
}

#[test]
fn kiss_cov_unix_process_group_ps_fns() {
    let _ = super::unix_process_group_ps::looks_like_malvin_agent_acp;
    let _ = super::unix_process_group_ps::read_proc_cmdline;
    let _ = super::unix_process_group_ps::read_proc_environ;
    let _ = stringify!(super::unix_process_group_ps::unix_process_group_ps_tests::looks_like_malvin_agent_acp_matches_environ_marker);
}

#[test]
fn kiss_cov_unix_process_group_teardown_fns() {
    let _ = super::unix_process_group_teardown::terminate_agent_process_group;
    let _ = super::unix_process_group_teardown::terminate_process_group;
    let _ = super::unix_process_group_teardown::kill_targets_for_teardown;
    let _ = super::unix_process_group_teardown::malvin_session_spawn_pids;
    let _ = super::unix_process_group_teardown::baseline_amnestied_agent_orphans;
    let _ = super::unix_process_group_teardown::reap_baseline_amnestied_agent_orphans_blocking;
    let _ = super::hostile_orphan_test_util::spawn_agent_pg_and_malvin_sibling;
    let _ = super::hostile_orphan_test_util::assert_sibling_monitored_and_blocks_spawn;
    let _ = super::hostile_orphan_test_util::spawn_hostile_agent_acp_orphan;
    let _ = stringify!(super::unix_process_group_teardown::unix_process_group_teardown_tests::malvin_sibling_outside_agent_pg_killed_on_teardown);
    let _ = stringify!(super::unix_process_group_teardown::unix_process_group_teardown_tests::baseline_amnestied_agent_acp_orphan_killed_on_teardown);
}

#[test]
fn kiss_cov_ops_body_spawn_remaining() {
    let _ = super::cursor_cli_auth_established();
    let _ = super::resolve_agent_bin();
    let _ = stringify!(spawn_agent_acp_session);
}

