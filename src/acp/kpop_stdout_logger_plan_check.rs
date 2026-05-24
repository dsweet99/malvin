//! `KPop` falsification tests for plan.md (stdout logger cleanup).

use crate::tool_summary::{ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines};
use crate::output::{
    AcpTeeDirection, AcpTeeLineFmt, AcpTeeStdoutEvent, acp_tee_log_line, is_log_timestamp_token,
    print_stdout_acp_tee_line_with_timestamp, print_stdout_acp_tool_summary_tee,
    set_stdout_log_path,
};
use serde_json::json;

#[test]
fn h2_read_without_path_suppresses_pending_stdout() {
    let pending = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_read_file",
            "kind": "read",
            "status": "pending",
            "title": "Read"
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&pending, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert!(
        lines.stdout.is_none(),
        "expected pending read with unknown path to be suppressed"
    );
}

#[test]
fn h3_edit_stdout_omits_line_number_for_edit_subject() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_edit_line",
            "kind": "edit",
            "status": "completed",
            "rawOutput": {
                "path": "src/foo.rs",
                "lineNumber": 12,
                "linesAdded": 1,
                "linesRemoved": 0
            }
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        !lines.log.contains(":12"),
        "log channel should omit line numbers for edits; log={}",
        lines.log
    );
    assert!(
        !stdout.contains(":12"),
        "stdout edit summary should omit line number; got {stdout:?}"
    );
    assert!(
        stdout.contains("src/foo.rs"),
        "stdout edit summary should include filename; got {stdout:?}"
    );
}

#[test]
fn h4_execute_command_newlines_are_escaped_in_stdout_summary() {
    let cmd = "python3 -c \"\nprint(1)\n\"";
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_nl",
            "kind": "execute",
            "status": "pending",
            "rawInput": {"command": cmd}
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_nl",
            "kind": "execute",
            "status": "completed",
            "rawOutput": {"exitCode": 0, "stdout": "", "stderr": ""}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.contains("\\n"),
        "expected escaped newline markers in stdout; got {stdout:?}"
    );
    assert!(
        !stdout.contains('\n'),
        "stdout command fragment should stay one physical line; got {stdout:?}"
    );
}

#[tokio::test]
async fn h6_trace_file_lines_include_timestamp() {
    use crate::acp::trace_line_write::TraceFileStdout;
    use crate::acp::{PromptTraceWriter, SessionUpdateChunkKind};

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("trace-ts.log");
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
        stdout_replacement: None,
        placeholder_emitted: false,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: true,
        iterable_closed_warned: false,
        work_dir: std::path::PathBuf::new(),
    };
    crate::acp::trace_file_write_line(
        &mut writer,
        "hello trace",
        Some(SessionUpdateChunkKind::Thought),
        TraceFileStdout {
            tee_stdout: false,
            stream_iterable_closed: None,
            tee_line_override: None,
            tee_line_display: None,
            ts: None,
        },
    )
    .await;
    drop(writer);
    let s = tokio::fs::read_to_string(&path).await.unwrap();
    let first_token = s.split_whitespace().next().unwrap_or("");
    assert!(
        crate::output::is_log_timestamp_token(first_token),
        "trace file line should start with timestamp; got {s:?}"
    );
}

#[test]
fn h7_live_stdout_log_both_tool_summary_and_thought_tee_timestamped() {
    let _guard = crate::output::STDOUT_LOG_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("stdout.log");
    set_stdout_log_path(Some(path.clone()));
    crate::output::init_stdout_style(false);

    print_stdout_acp_tool_summary_tee(
        &AcpTeeStdoutEvent {
            direction: AcpTeeDirection::FromAgent,
            who: "<check_plan",
            line: "Run rg foo · 1ms · ✓",
            ts: "20260413.121314.015",
            emit_stdout_markdown: false,
            dim_payload: false,
        },
        "Run rg foo · 1ms · ✓",
    );
    print_stdout_acp_tee_line_with_timestamp(&AcpTeeStdoutEvent {
        direction: AcpTeeDirection::FromAgent,
        who: "<check_plan",
        line: "[Verifying proposal]",
        ts: "20260413.121314.015",
        emit_stdout_markdown: false,
        dim_payload: true,
    });
    set_stdout_log_path(None);

    let text = std::fs::read_to_string(path).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "expected two log lines; got {text:?}");
    assert!(
        is_log_timestamp_token(lines[0].split_whitespace().next().unwrap_or("")),
        "tool summary line should be timestamped; got {:?}",
        lines[0]
    );
    assert!(
        is_log_timestamp_token(lines[1].split_whitespace().next().unwrap_or("")),
        "thought tee line should be timestamped; got {:?}",
        lines[1]
    );
    let plain = "Run rg foo · 1ms · ✓";
    let ts = "20260413.121314.015";
    let ctx = AcpTeeLineFmt {
        ts,
        direction: AcpTeeDirection::FromAgent,
        who: "<check_plan",
        line: plain,
        dim_payload: false,
    };
    assert_eq!(lines[0], acp_tee_log_line(&ctx));
    assert!(
        lines[0].contains(plain),
        "stdout.log tool summary should carry plain payload; got {:?}",
        lines[0]
    );
}

#[test]
fn h8_path_in_start_raw_input_is_retained_on_pending_update() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_read_path",
            "kind": "read",
            "status": "pending",
            "rawInput": {"path": "src/acp/tool_summary.rs"}
        }}
    });
    let pending = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_read_path",
            "kind": "read",
            "status": "pending"
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&pending, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.contains("tool_summary.rs"),
        "path from start rawInput should be persisted in tracker; got {stdout:?}"
    );
}
