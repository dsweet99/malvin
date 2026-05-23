use crate::tool_summary::{ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines};
use serde_json::json;

async fn write_parsed_trace_line(
    writer: &mut crate::acp::PromptTraceWriter,
    coalesce: &mut crate::acp::TraceChunkCoalescer,
    raw_line: &str,
) {
    let parsed = serde_json::from_str(raw_line).unwrap();
    crate::acp::trace_line_write::write_trace_line_coalesced(
        writer,
        coalesce,
        crate::acp::trace_line_write::WriteTraceLineCoalescedOpts {
            parsed: Some(&parsed),
            raw_line,
            tee_stdout: false,
        },
    )
    .await;
}

#[tokio::test]
async fn coalesced_tool_done_omits_full_stdout_in_trace() {
    let start_line = r#"{"method":"session/update","params":{"update":{"sessionUpdate":"tool_call","toolCallId":"tool_done","kind":"execute","status":"pending","rawInput":{"command":"false"}}}}"#;
    let done_line = r#"{"method":"session/update","params":{"update":{"sessionUpdate":"tool_call_update","toolCallId":"tool_done","kind":"execute","status":"completed","rawOutput":{"exitCode":101,"stdout":"warning: unused import\n","stderr":"error: could not compile\n"}}}}"#;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("tool-done.log");
    let file = tokio::fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(&path)
        .await
        .unwrap();
    let mut writer = crate::acp::PromptTraceWriter {
        file,
        who: "kpop".to_string(),
        plain_lines: false,
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
        iterable_closed_warned: false,
    };
    let mut coalesce = crate::acp::TraceChunkCoalescer::default();
    write_parsed_trace_line(&mut writer, &mut coalesce, start_line).await;
    write_parsed_trace_line(&mut writer, &mut coalesce, done_line).await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    assert!(s.contains("[tool] done") && s.contains("exit=101"));
    assert!(!s.contains("unused import"));
    assert!(s.contains("error="));
}

#[test]
fn long_command_uses_middle_ellipsis() {
    let long_cmd = format!(
        "cd {} && cargo clippy && cargo nextest run",
        "a/".repeat(30)
    );
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_long",
            "kind": "execute",
            "status": "pending",
            "rawInput": {"command": long_cmd}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(lines.log.contains("..."));
    assert!(lines.log.chars().count() < 200);
}
