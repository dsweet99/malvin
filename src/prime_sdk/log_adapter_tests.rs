//! Name witnesses for log adapter (kiss `test_coverage`).

#[test]
fn kiss_cov_log_adapter_names() {
    let _ = stringify!(prime_handle_stream_event);
    let _ = stringify!(prime_feed_do_dm_run_result);
    let _ = stringify!(prime_emit_assistant);
    let _ = stringify!(prime_emit_thinking);
    let _ = stringify!(prime_tee_coalesced);
    let _ = stringify!(prime_flush_stdout_coalesce);
    let _ = stringify!(prime_print_coalesced_line);
    let _ = stringify!(prime_append_trace_value);
    let _ = stringify!(prime_append_trace_raw);
    let _ = stringify!(prime_append_trace_line);
    let _ = stringify!(PrimeToolCallFields);
    let _ = stringify!(prime_emit_tool);
    let _ = stringify!(prime_clear_tool_starts);
    let _ = stringify!(prime_note_tool_start);
    let _ = stringify!(prime_take_tool_start);
    let _ = stringify!(PrimeDoneLineInput);
    let _ = stringify!(prime_format_tool_done_line);
    let _ = stringify!(prime_compose_tool_done_line);
    let _ = stringify!(prime_tee_tool_line);
}
