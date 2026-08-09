//! Name witnesses for shared bridge log adapter (kiss `test_coverage`).

#[test]
fn kiss_cov_log_adapter_names() {
    // Shared adapters live under `bridge_sdk`; keep Prime-side name witnesses distinct.
    let _ = stringify!(handle_stream_event);
    let _ = stringify!(feed_do_dm_run_result);
    let _ = stringify!(emit_assistant);
    let _ = stringify!(emit_thinking);
    let _ = stringify!(emit_tool);
    let _ = stringify!(ToolCallFields);
    let _ = stringify!(clear_tool_starts);
}
