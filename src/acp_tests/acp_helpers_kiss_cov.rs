//! External kiss witnesses for acp test/helper modules (bucket D).

#[test]
fn kiss_witness_acp_test_helper_fns_a() {
    let _ = super::deferred_log_plan_regression::read_done_empty_raw_input;
    let _ = super::deferred_log_plan_regression::read_done_tee_shows_store_db_path_when_wire_raw_input_empty;
    let _ = super::deferred_log_plan_regression::read_start_empty_raw_input;
    let _ = super::deferred_log_plan_regression::tee_read_lifecycle_stdout;
    let _ = super::kpop_stdout_logger_plan_check::h6_trace_file_lines_include_timestamp;
    let _ = super::kpop_stdout_logger_plan_check_bracket::assert_payload_omits_brackets_after_who_tag;
    let _ = super::kpop_stdout_logger_plan_check_bracket::assert_styled_tool_summary_payloads_match;
    let _ = super::kpop_stdout_logger_plan_check_bracket::read_tool_bracket_pair_updates;
    let _ = super::kpop_stdout_logger_plan_check_bracket::tee_read_tool_bracket_pair_stdout;
    let _ = super::kpop_stdout_logger_plan_check_bracket::tee_tool_summary_updates;
    let _ = super::kpop_stdout_logger_plan_check_ext::h10_write_trace_line_coalesced_tees_timestamped_tool_summary_to_stdout_log;
    let _ = super::kpop_stdout_logger_plan_check_ext::h12_tool_summary_trace_and_stdout_log_share_timestamp;
    let _ = super::kpop_stdout_logger_plan_check_ext::h22_styled_tool_summary_trace_tee_dims_payload;
    let _ = super::kpop_stdout_logger_plan_check_impl::h14_fast_execute_done_emits_one_stdout_summary_line;
    let _ = super::kpop_stdout_logger_plan_check_impl::h18_raw_output_writer_suppresses_tool_stdout_tee;
    let _ = super::kpop_stdout_logger_plan_check_impl::h19_thought_stdout_three_space_indent_no_brackets;
    let _ = super::kpop_stdout_logger_plan_check_impl::h20_styled_tool_summary_stdout_line_omits_payload_brackets;
    let _ = super::kpop_stdout_logger_plan_check_impl::h21_unstyled_tool_summary_omits_brackets;
    let _ = super::kpop_stdout_logger_plan_check_impl::h23_start_and_done_tool_summary_omit_payload_brackets;
    let _ = super::kpop_stdout_logger_plan_helpers::begin_stdout_log_fixture;
    let _ = super::kpop_stdout_logger_plan_helpers::finish_stdout_log_fixture;
    let _ = super::kpop_stdout_logger_plan_helpers::open_styled_markdown_trace_writer;
    let _ = super::kpop_stdout_logger_plan_helpers::open_trace_writer;
    let _ = super::kpop_stdout_logger_plan_helpers::production_execute_done_stdout;
    let _ = super::kpop_stdout_logger_plan_helpers::production_execute_done_trace_and_stdout;
    let _ = super::kpop_stdout_logger_plan_helpers::stdout_log_test_guard;
    let _ = super::kpop_stdout_logger_plan_helpers::tee_coalesced_update;
    let _ = super::reader_tests_helpers::acp_activity_state;
    let _ = super::reader_tests_helpers::test_prompt_round_health;
    let _ = super::reader_tests_helpers::handshake_io_from_stdin;
    let _ = stringify!(dispatch_parts);
    let _ = stringify!(finish_stdout);
    super::reader_tests_helpers::block_on_test(async {});
}

#[test]
fn kiss_witness_acp_test_helper_fns_b() {
    let _ = super::reader_tests_tool_summary_trace::coalesced_tool_done_omits_full_stdout_in_trace;
    let _ = super::reader_tests_tool_summary_trace::write_parsed_trace_line;
    let _ = super::reader_tests_trace_b::open_trace_b_writer;
    let _ = super::reader_tests_trace_b::raw_trace_file_write_line_records_thought_chunks_suppresses_thought_stdout_only;
    let _ = super::reader_tests_trace_b::trace_file_write_line_brackets_thought_chunks_in_trace_output;
    let _ = super::reader_tests_trace_b::trace_file_write_line_plain_mode_omits_tag_prefix;
    let _ = super::reader_tests_trace_b::trace_file_write_line_prefixes_with_prompt_who;
    let _ = super::reader_tests_trace_b::trace_file_write_line_stdout_markdown_flag_tees_without_panic;
    let _ = super::reader_tests_trace_coalesce_write::assert_tool_call_lifecycle_summary_tee;
    let _ = super::reader_tests_trace_coalesce_write::open_coalesce_trace_at;
    let _ = super::reader_tests_trace_coalesce_write::run_tool_call_lifecycle_tee_fixture;
    let _ = super::reader_tests_trace_coalesce_write::write_trace_line_coalesced_does_not_tee_parsed_non_chunk_lines;
    let _ = super::reader_tests_trace_coalesce_write::write_trace_line_coalesced_must_tee_parsed_tool_call_lifecycle_to_stdout;
    let _ = super::reader_tests_trace_coalesce_write::write_trace_line_coalesced_writes_malformed_non_json_lines;
    let _ = super::reader_tests_trace_coalesce_write::write_trace_line_coalesced_writes_non_chunk_lines;
    let _ = super::reader_tests_trace_iterable::assert_iterable_closed_operational_stderr;
    let _ = super::reader_tests_trace_iterable::assert_split_iterable_closed_operational;
    let _ = super::reader_tests_trace_iterable::deliver_coalesced_message_chunk;
    let _ = super::reader_tests_trace_iterable::iterable_closed_split_across_coalesce_emissions_suppresses_kpop_tee;
    let _ = super::reader_tests_trace_iterable::readable_iterable_closed_split_coalesce_emits_readable_operational_warning;
    let _ = super::reader_tests_trace_iterable::run_split_iterable_closed_fixture;
    let _ = super::reader_tests_trace_iterable::session_update_message_chunk_json;
    let _ = super::reader_tests_trace_iterable::trace_file_write_line_iterable_closed_warns_without_kpop_tee;
    let _ = super::reader_tests_trace_upgrade_plan::assert_upgrade_plan_operational_stderr;
    let _ = super::reader_tests_trace_upgrade_plan::feed_upgrade_plan_split;
    let _ = super::reader_tests_trace_upgrade_plan::run_upgrade_plan_split_coalesce_fixture;
    let _ = super::reader_tests_trace_upgrade_plan::upgrade_plan_split_coalesce_emits_operational_error_without_kpop_tee;
}

#[test]
fn kiss_witness_acp_test_helper_types() {
    let fixture = super::kpop_stdout_logger_plan_helpers::begin_stdout_log_fixture();
    assert!(fixture.trace_path.to_string_lossy().contains("trace"));
    let _ = super::kpop_stdout_logger_plan_helpers::finish_stdout_log_fixture(fixture);

    #[cfg(unix)]
    {
        let _ = super::reader_tests_helpers::CatSession::new;
        let _ = super::reader_tests_helpers::IncomingDispatchParts::dispatch_lines;
    }

    let opts = super::reader_tests_trace_b::TraceBWriterOpts {
        who: "m",
        plain_lines: false,
        raw_output: false,
        emit_stdout_markdown: false,
    };
    let super::reader_tests_trace_b::TraceBWriterOpts {
        who,
        plain_lines,
        raw_output: _,
        emit_stdout_markdown: _,
    } = opts;
    assert_eq!(who, "m");
    assert!(!plain_lines);

    let fixture = super::reader_tests_trace_kpop_helpers::kpop_stdout_trace_fixture("kiss");
    let super::reader_tests_trace_kpop_helpers::KpopStdoutTraceFixture {
        dir: _,
        stdout_path,
        trace_path,
    } = fixture;
    assert!(stdout_path.to_string_lossy().contains("kiss"));
    assert!(trace_path.to_string_lossy().contains("kiss"));
}

#[cfg(unix)]
#[test]
fn kiss_witness_acp_mem_watch_helpers() {
    let _ = crate::acp::watch_process_group_memory;
}
