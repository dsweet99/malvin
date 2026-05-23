async fn write_tool_summary_trace_line(
    trace_file: &mut PromptTraceWriter,
    coalesce: &mut TraceChunkCoalescer,
    parsed: &serde_json::Value,
    tee_stdout: bool,
) -> bool {
    let Some(summary) =
        tool_summary_lines(parsed, &mut coalesce.tool_tracker, ToolSummaryDetail::Stdout)
    else {
        return false;
    };
    let ts = crate::output::timestamp_now_string();
    trace_file_write_line(
        trace_file,
        &summary.log,
        None,
        TraceFileStdout {
            tee_stdout: false,
            stream_iterable_closed: None,
            tee_line_override: None,
            tee_line_display: None,
            ts: Some(&ts),
        },
    )
    .await;
    if !tee_stdout {
        return true;
    }
    let ctx = TraceTeeStdoutCtx {
        tee_stdout: true,
        kind: None,
        ts: &ts,
    };
    for plain in [
        summary.stdout_deferred.as_deref(),
        summary.stdout.as_deref(),
    ]
    .into_iter()
    .flatten()
    {
        let display = tool_summary_stdout_display(plain);
        trace_tee_stdout_line(trace_file, plain, Some(display.as_str()), &ctx);
    }
    true
}
