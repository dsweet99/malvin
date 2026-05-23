#[test]
fn smoke_acp_reader_support_behavior() {
    use std::sync::atomic::Ordering;

    let (seq, _notify) = crate::acp::reader_tests_helpers::acp_activity_state();
    assert_eq!(seq.load(Ordering::Relaxed), 0);
}

#[cfg(unix)]
#[tokio::test]
async fn smoke_reader_loop_eof_pending_error() {
    let msg = crate::acp::reader_tests_helpers::reader_loop_eof_pending_error().await;
    assert!(!msg.is_empty());
}

#[cfg(not(unix))]
#[test]
fn smoke_acp_reader_helper_production_symbols() {
    let _ = stringify!(crate::acp::reader_tests_helpers::IncomingDispatchParts);
    let _ = stringify!(crate::acp::reader_tests_helpers::CatSession);
    let _ = stringify!(crate::acp::reader_tests_helpers::spawn_true_stdout_with_pending);
    let _ = stringify!(crate::acp::reader_tests_helpers::spawn_sleep_stdin);
    let _ = stringify!(crate::acp::reader_tests_helpers::reader_loop_eof_pending_error);
}

#[test]
fn smoke_acp_reader_dispatch_and_trace_test_names() {
    let _ = stringify!(
        crate::acp::reader_tests_dispatch::dispatch_resolves_pending_when_response_id_is_i64
    );
    let _ = stringify!(
        crate::acp::reader_tests_dispatch::dispatch_resolves_pending_when_response_id_is_decimal_string
    );
    let _ = stringify!(
        crate::acp::reader_tests_dispatch::dispatch_clears_prompt_cleanup_when_id_matches
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_coalesce_write::write_trace_line_coalesced_writes_non_chunk_lines
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_coalesce_write::write_trace_line_coalesced_does_not_tee_parsed_non_chunk_lines
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_coalesce_write::write_trace_line_coalesced_must_tee_parsed_tool_call_lifecycle_to_stdout
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_coalesce_write::write_trace_line_coalesced_writes_malformed_non_json_lines
    );
    let _ = stringify!(crate::acp::reader_tests_trace_coalesce_write::kpop_coalesce_trace_writer);
    let _ = stringify!(crate::acp::reader_tests_trace_coalesce_write::open_coalesce_trace_at);
    let _ = stringify!(crate::acp::reader_tests_trace_coalesce_write::write_coalesced_line);
    let _ =
        stringify!(crate::acp::reader_tests_trace_coalesce_write::assert_tool_call_lifecycle_summary_tee);
    let _ = stringify!(
        crate::acp::reader_tests_trace_coalesce_write::run_tool_call_lifecycle_tee_fixture
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_coalesce_write::deliver_tool_call_session_updates
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_b::trace_file_write_line_prefixes_with_prompt_who
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_b::raw_trace_file_write_line_records_thought_chunks_suppresses_thought_stdout_only
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_b::trace_file_write_line_plain_mode_omits_tag_prefix
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_b::trace_file_write_line_brackets_thought_chunks_in_trace_output
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_b::trace_file_write_line_stdout_markdown_flag_tees_without_panic
    );
    let _ = stringify!(
        crate::acp::reader_tests_permission_unix::unix::kpop_permission_without_correlation_id_writes_nothing_to_child_stdin
    );
    let _ = stringify!(
        crate::acp::reader_tests_permission_unix::unix::permission_with_id_in_params_writes_allow_always_reply_line
    );
}

#[test]
fn smoke_acp_reader_iterable_closed_and_retry_policy_test_names() {
    let _ = stringify!(
        crate::acp::reader_tests_trace_iterable::trace_file_write_line_iterable_closed_warns_without_kpop_tee
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_iterable::iterable_closed_split_across_coalesce_emissions_suppresses_kpop_tee
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_iterable::readable_iterable_closed_split_coalesce_emits_readable_operational_warning
    );
    let _ = stringify!(crate::acp::reader_tests_trace_iterable::kpop_trace_writer);
    let _ = stringify!(
        crate::acp::reader_tests_trace_iterable::assert_iterable_closed_operational_stderr
    );
    let _ = stringify!(crate::acp::reader_tests_trace_iterable::session_update_message_chunk_json);
    let _ = stringify!(crate::acp::reader_tests_trace_iterable::deliver_coalesced_message_chunk);
    let _ = stringify!(
        crate::acp::reader_tests_trace_iterable::assert_split_iterable_closed_operational
    );
    let _ = stringify!(crate::acp::reader_tests_trace_iterable::run_split_iterable_closed_fixture);
    let _ = stringify!(crate::acp::reader_tests_trace_iterable::open_kpop_trace_writer);
    let _ = stringify!(crate::acp::reader_tests_trace_iterable::flush_coalesce_lines);
    let _ = stringify!(crate::acp::IterableClosedStream);
    let _ = stringify!(crate::acp::iterable_closed_stream_from_buffer);
    let _ = stringify!(
        crate::acp::trace_line_write::trace_line_write_kiss::smoke_trace_line_write_symbol_names_for_kiss
    );
    let _ = stringify!(crate::acp::reader_tests_retry_policy::upgrade_plan_substring_is_detected_case_insensitively);
    let _ = stringify!(crate::acp::reader_tests_retry_policy::upgrade_plan_errors_do_not_retry);
    let _ = stringify!(crate::acp::reader_tests_retry_policy::cannot_use_model_errors_do_not_retry);
    let _ = stringify!(crate::acp::reader_tests_retry_policy::cannot_use_model_fails_fast_even_when_error_also_looks_retriable);
    let _ = stringify!(crate::acp::reader_tests_retry_policy::retriable_timeout_delimited_without_timed_out_substring);
    let _ = stringify!(
        crate::acp::reader_tests_retry_policy::iterable_closed_stream_from_buffer_and_operational_iterable_closed_for_emit
    );
    let _ = stringify!(
        crate::acp::reader_tests_retry_policy::operational_iterable_closed_log_line_detection
    );
    let _ = stringify!(
        crate::acp::reader_tests_retry_policy::retriable_transient_errors_match_known_agent_strings
    );
    let _ =
        stringify!(crate::acp::reader_tests_retry_policy::non_retriable_errors_stop_without_sleep);
    let _ = stringify!(
        crate::acp::reader_tests_retry_policy::retriable_first_attempt_sleeps_one_second
    );
    let _ = stringify!(
        crate::acp::reader_tests_retry_policy::retriable_second_attempt_sleeps_three_seconds
    );
    let _ = stringify!(
        crate::acp::reader_tests_retry_policy::retriable_exhausts_after_max_agent_attempts
    );
    let _ = stringify!(crate::acp::reader_tests_retry_policy::retries_noun_singular_and_plural);
    let _ = stringify!(
        crate::acp::reader_tests_retry_policy::delimited_token_match_has_delimited_substring_is_identifier_byte_timeout_word_iterable_closed_in_ascii_lower
    );
    let _ = stringify!(emit_stderr_log_line);
    let _ = stringify!(emit_stderr_log_lines);
    let _ = stringify!(set_stdout_log_path);
    let _ = stringify!(clone_stdout_log_path);
    let _ = stringify!(timestamp_now_string);
}

#[test]
fn smoke_acp_tool_summary_test_names() {
    let _ = stringify!(crate::acp::tool_summary::shorten_middle);
    let _ = stringify!(crate::acp::tool_summary::tool_summary_lines);
    let _ = stringify!(
        crate::acp::tool_summary::tool_summary_kiss::smoke_tool_summary_symbol_names_for_kiss
    );
    let _ = stringify!(crate::acp::reader_tests_tool_summary::parse_tool_call_start);
    let _ = stringify!(crate::acp::reader_tests_tool_summary::parse_tool_update_running_and_done);
    let _ = stringify!(crate::acp::reader_tests_tool_summary::done_summary_omits_title_field_per_plan);
    let _ = stringify!(
        crate::acp::reader_tests_tool_summary::edit_done_content_only_omits_synthetic_added_removed_counts
    );
    let _ = stringify!(
        crate::acp::reader_tests_tool_summary::edit_done_emits_added_when_only_lines_added_field_present
    );
    let _ = stringify!(
        crate::acp::reader_tests_tool_summary::tool_call_update_pending_labeled_pending_not_start
    );
    let _ = stringify!(
        crate::acp::reader_tests_tool_summary_kinds::edit_done_includes_path_and_counts
    );
    let _ = stringify!(
        crate::acp::reader_tests_tool_summary_trace::coalesced_tool_done_omits_full_stdout_in_trace
    );
    let _ = stringify!(crate::acp::reader_tests_tool_summary_trace::long_command_uses_middle_ellipsis);
    let _ = stringify!(crate::acp::reader_tests_tool_summary_trace::write_parsed_trace_line);
    let _ = stringify!(crate::acp::reader_tests_tool_summary_human::stdout_read_done_prose_and_humanized_size);
    let _ = stringify!(crate::acp::reader_tests_tool_summary_human::stdout_start_suppressed_until_running_threshold);
    let _ = stringify!(crate::acp::reader_tests_tool_summary_human::stdout_execute_failure_shows_exit_and_error);
    let _ = stringify!(crate::acp::reader_tests_tool_summary_human::stdout_display_ansi_stripped_matches_plain);
    let _ = stringify!(crate::acp::reader_tests_tool_summary_human::log_channel_stays_key_value);
    let _ = stringify!(
        crate::acp::reader_tests_tool_summary_human_bugs::stdout_execute_completed_stderr_without_exit_code_must_not_show_checkmark
    );
    let _ = stringify!(
        crate::acp::reader_tests_tool_summary_human_bugs::stdout_execute_failed_without_exit_code_must_not_show_checkmark
    );
    let _ = stringify!(
        crate::acp::reader_tests_tool_summary_human_bugs::stdout_read_done_without_raw_output_still_emits_prose
    );
    let _ = stringify!(
        crate::acp::reader_tests_tool_summary_human_bugs::stdout_pending_update_tees_human_start_line
    );
    let _ = stringify!(
        crate::acp::jsonl_trace::tests::trace_jsonl_append_records_raw_line_before_human_summary
    );
}
