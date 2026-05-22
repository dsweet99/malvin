use crate::acp::*;

fn kpop_coalesce_trace_writer(file: tokio::fs::File) -> PromptTraceWriter {
    PromptTraceWriter {
        file,
        who: "kpop".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
        iterable_closed_warned: false,
    }
}

async fn open_coalesce_trace_at(
    path: &std::path::Path,
) -> (PromptTraceWriter, TraceChunkCoalescer) {
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(path)
        .await
        .unwrap();
    (
        kpop_coalesce_trace_writer(file),
        TraceChunkCoalescer::default(),
    )
}

async fn write_coalesced_line(
    writer: &mut PromptTraceWriter,
    coalesce: &mut TraceChunkCoalescer,
    opts: crate::acp::trace_line_write::WriteTraceLineCoalescedOpts<'_>,
) {
    crate::acp::trace_line_write::write_trace_line_coalesced(writer, coalesce, opts).await;
}

async fn deliver_tool_call_session_updates(
    writer: &mut PromptTraceWriter,
    coalesce: &mut TraceChunkCoalescer,
) {
    let tool_call_line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"85738996-e440-40f3-9055-75e4fcfc934e","update":{"sessionUpdate":"tool_call","toolCallId":"tool_026fbf9d-cf5b-4010-b626-8e2547fa6b4","title":"`ls -ltr ./_malvin`","kind":"execute","status":"pending","rawInput":{"command":"ls -ltr ./_malvin"}}}}"#;
    let parsed = serde_json::from_str(tool_call_line).unwrap();
    write_coalesced_line(
        writer,
        coalesce,
        crate::acp::trace_line_write::WriteTraceLineCoalescedOpts {
            parsed: Some(&parsed),
            raw_line: tool_call_line,
            tee_stdout: true,
        },
    )
    .await;
    let tool_update_line = r#"{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"85738996-e440-40f3-9055-75e4fcfc934e","update":{"sessionUpdate":"tool_call_update","toolCallId":"tool_026fbf9d-cf5b-4010-b626-8e2547fa6b4","status":"in_progress"}}}"#;
    let parsed_update = serde_json::from_str(tool_update_line).unwrap();
    write_coalesced_line(
        writer,
        coalesce,
        crate::acp::trace_line_write::WriteTraceLineCoalescedOpts {
            parsed: Some(&parsed_update),
            raw_line: tool_update_line,
            tee_stdout: true,
        },
    )
    .await;
}

fn assert_tool_call_lifecycle_summary_tee(trace: &str, stdout: &str) {
    assert!(
        !trace.contains("sessionUpdate"),
        "tool lifecycle must be summarized in prompt trace, not raw JSON; got {trace:?}"
    );
    assert!(
        trace.contains("[tool] start") && trace.contains("[tool] running"),
        "prompt trace must include tool start and running summaries; got {trace:?}"
    );
    assert!(
        stdout.contains("[tool] start") && stdout.contains("[tool] running"),
        "tool summaries must tee to stdout when tee_stdout is enabled; got {stdout:?}"
    );
    assert!(
        !stdout.contains("tool_call_update"),
        "stdout must not contain raw tool JSON; got {stdout:?}"
    );
}

async fn run_tool_call_lifecycle_tee_fixture() -> (String, String) {
    let tmp = tempfile::tempdir().unwrap();
    let stdout_path = tmp.path().join("stdout.log");
    let trace_path = tmp.path().join("tool-trace.log");
    crate::output::set_stdout_log_path(Some(stdout_path.clone()));
    crate::output::init_stdout_style(true);

    let (mut writer, mut coalesce) = open_coalesce_trace_at(&trace_path).await;
    deliver_tool_call_session_updates(&mut writer, &mut coalesce).await;
    drop(writer);
    crate::output::set_stdout_log_path(None);

    let trace = tokio::fs::read_to_string(&trace_path).await.unwrap();
    let stdout = std::fs::read_to_string(&stdout_path).unwrap_or_default();
    (trace, stdout)
}

#[tokio::test]
async fn write_trace_line_coalesced_writes_non_chunk_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("coalesce-trace.log");
    let (mut writer, mut coalesce) = open_coalesce_trace_at(&path).await;
    let parsed = serde_json::json!({"jsonrpc":"2.0","id":1,"result":{"ok":true}});
    write_coalesced_line(
        &mut writer,
        &mut coalesce,
        crate::acp::trace_line_write::WriteTraceLineCoalescedOpts {
            parsed: Some(&parsed),
            raw_line: r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#,
            tee_stdout: false,
        },
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(
        s.contains(r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#),
        "non-chunk ACP lines should be preserved in trace output"
    );
}

#[tokio::test]
async fn write_trace_line_coalesced_does_not_tee_parsed_non_chunk_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("coalesce-trace-no-tee.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = PromptTraceWriter {
        file,
        who: "kpop".to_string(),
        plain_lines: false,
        stdout_replacement: Some("<suppressed>"),
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
        iterable_closed_warned: false,
    };
    let mut coalesce = TraceChunkCoalescer::default();
    let parsed = serde_json::json!({"jsonrpc":"2.0","id":1,"result":{"ok":true}});
    write_coalesced_line(
        &mut writer,
        &mut coalesce,
        crate::acp::trace_line_write::WriteTraceLineCoalescedOpts {
            parsed: Some(&parsed),
            raw_line: r#"{"jsonrpc":"2.0","id":1,"result":{"ok":true}}"#,
            tee_stdout: true,
        },
    )
    .await;
    assert!(
        !writer.placeholder_emitted,
        "parsed non-chunk ACP protocol lines must not be tee'd to stdout"
    );
}

#[tokio::test]
async fn write_trace_line_coalesced_must_tee_parsed_tool_call_lifecycle_to_stdout() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (trace, stdout) = run_tool_call_lifecycle_tee_fixture().await;
    assert_tool_call_lifecycle_summary_tee(&trace, &stdout);
}

#[tokio::test]
async fn write_trace_line_coalesced_writes_malformed_non_json_lines() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("coalesce-trace-malformed.log");
    let (mut writer, mut coalesce) = open_coalesce_trace_at(&path).await;
    write_coalesced_line(
        &mut writer,
        &mut coalesce,
        crate::acp::trace_line_write::WriteTraceLineCoalescedOpts {
            parsed: None,
            raw_line: "not-json {{{",
            tee_stdout: false,
        },
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(
        s.contains("not-json {{{"),
        "malformed non-JSON ACP lines should still be preserved in trace output"
    );
}
