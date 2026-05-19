#[test]
fn kiss_stringify_acp_reader_and_support_units() {
    let _ = stringify!(crate::acp::pair::ReviewerPromptPair);
    let _ = stringify!(crate::acp::reader_tests_helpers::acp_activity_state);
    let _ = stringify!(
        crate::acp::reader_tests_dispatch::dispatch_resolves_pending_when_response_id_is_i64
    );
    let _ = stringify!(
        crate::acp::reader_tests_dispatch::dispatch_resolves_pending_when_response_id_is_decimal_string
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_a::write_trace_line_coalesced_writes_non_chunk_lines
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_a::write_trace_line_coalesced_does_not_tee_parsed_non_chunk_lines
    );
    let _ = stringify!(
        crate::acp::reader_tests_trace_a::write_trace_line_coalesced_writes_malformed_non_json_lines
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
    let _ = stringify!(
        crate::acp::reader_tests_dispatch::dispatch_clears_prompt_cleanup_when_id_matches
    );
}

#[test]
fn kiss_stringify_acp_ops_inline_tests_units() {
    let _ = stringify!(crate::acp::ops_inline_tests::write_path_executable);
}
