//! Plan implementation regression tests (h14–h21).

use super::kpop_stdout_logger_plan_helpers::{
    begin_stdout_log_fixture, execute_tool_done_json, execute_tool_json,
    finish_stdout_log_fixture, open_styled_markdown_trace_writer, open_trace_writer,
    stdout_log_test_guard, tee_coalesced_update,
};
use crate::acp::trace_line_write::TraceFileStdout;
use crate::acp::write_tool_summary_trace_line;
use crate::output::is_log_timestamp_token;
use crate::tool_summary::{
    relativize_tool_path, ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines,
};
use serde_json::json;

#[tokio::test]
async fn h14_fast_execute_done_emits_one_stdout_summary_line() {
    let _guard = stdout_log_test_guard();
    let fixture = begin_stdout_log_fixture();
    let (mut writer, mut coalesce) =
        open_styled_markdown_trace_writer(&fixture.trace_path, fixture.tmp.path()).await;
    write_tool_summary_trace_line(
        &mut writer,
        &mut coalesce,
        &execute_tool_json("tool_fast", "pending", "echo hi"),
        true,
    )
    .await;
    write_tool_summary_trace_line(
        &mut writer,
        &mut coalesce,
        &execute_tool_done_json("tool_fast"),
        true,
    )
    .await;
    drop(writer);
    let stdout = finish_stdout_log_fixture(fixture);
    let tool_lines: Vec<_> = stdout.lines().filter(|l| l.contains("Run ")).collect();
    assert_eq!(tool_lines.len(), 1, "got {stdout:?}");
    assert!(tool_lines[0].contains('✓'));
}

#[test]
fn h15_read_done_shows_path_from_start_raw_input() {
    let tmp = tempfile::tempdir().unwrap();
    let path = "src/acp/trace_line_write.rs";
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_read_path_done",
            "kind": "read",
            "status": "pending",
            "rawInput": {"path": path}
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_read_path_done",
            "kind": "read",
            "status": "completed",
            "rawOutput": {"content": "x"}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tracker.set_work_dir(tmp.path().to_path_buf());
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(stdout.contains("trace_line_write.rs"), "got {stdout:?}");
    assert!(!stdout.contains("Read file ·"), "got {stdout:?}");
}

#[test]
fn h16_search_done_includes_query_from_start_raw_input() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_search_q",
            "kind": "search",
            "status": "pending",
            "rawInput": {"query": "rg foo"}
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_search_q",
            "kind": "search",
            "status": "completed",
            "rawOutput": {"totalMatches": 0}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(stdout.contains("rg foo"), "got {stdout:?}");
}

#[test]
fn h17_relativize_tool_path_under_work_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let abs = tmp.path().join("src/foo.rs");
    let rel = relativize_tool_path(abs.to_str().unwrap(), Some(tmp.path()));
    assert_eq!(rel, "./src/foo.rs");
}

#[tokio::test]
async fn h18_raw_output_writer_suppresses_tool_stdout_tee() {
    let _guard = stdout_log_test_guard();
    let fixture = begin_stdout_log_fixture();
    let (mut writer, mut coalesce) =
        open_styled_markdown_trace_writer(&fixture.trace_path, fixture.tmp.path()).await;
    writer.raw_output = true;
    write_tool_summary_trace_line(
        &mut writer,
        &mut coalesce,
        &execute_tool_done_json("tool_do_suppress"),
        true,
    )
    .await;
    drop(writer);
    let stdout = finish_stdout_log_fixture(fixture);
    assert!(stdout.trim().is_empty(), "got {stdout:?}");
}

#[tokio::test]
async fn h19_thought_stdout_three_space_indent_no_brackets() {
    let _guard = stdout_log_test_guard();
    let mut fixture = begin_stdout_log_fixture();
    fixture.trace_path = fixture.tmp.path().join("trace-thought.log");
    let (mut writer, _) =
        open_styled_markdown_trace_writer(&fixture.trace_path, fixture.tmp.path()).await;
    crate::acp::trace_file_write_line(
        &mut writer,
        "internal reasoning",
        Some(crate::acp::SessionUpdateChunkKind::Thought),
        TraceFileStdout {
            tee_stdout: true,
            stream_iterable_closed: None,
            stream_upgrade_plan: false,
            tee_line_override: None,
            tee_line_display: None,
            ts: Some("20260413.121314.015"),
        },
    )
    .await;
    drop(writer);
    let trace = tokio::fs::read_to_string(&fixture.trace_path).await.unwrap();
    let stdout = finish_stdout_log_fixture(fixture);
    assert!(trace.contains("[internal reasoning]"), "got {trace:?}");
    assert!(stdout.contains("   internal reasoning"), "got {stdout:?}");
    assert!(!stdout.contains("[internal reasoning]"), "got {stdout:?}");
}

#[tokio::test]
async fn h20_styled_tool_summary_stdout_line_omits_payload_brackets() {
    let _guard = stdout_log_test_guard();
    let fixture = begin_stdout_log_fixture();
    let (mut writer, mut coalesce) =
        open_styled_markdown_trace_writer(&fixture.trace_path, fixture.tmp.path()).await;
    write_tool_summary_trace_line(
        &mut writer,
        &mut coalesce,
        &execute_tool_json("tool_colon", "pending", "echo hi"),
        true,
    )
    .await;
    write_tool_summary_trace_line(
        &mut writer,
        &mut coalesce,
        &execute_tool_done_json("tool_colon"),
        true,
    )
    .await;
    drop(writer);
    let stdout = finish_stdout_log_fixture(fixture);
    let line = stdout.lines().find(|l| l.contains("Run ")).expect("tool summary");
    assert!(is_log_timestamp_token(line.split_whitespace().next().unwrap_or("")));
    assert!(!line.contains("[Run"), "got {line:?}");
    assert!(line.contains("Run "), "got {line:?}");
    assert!(line.contains("echo hi"), "got {line:?}");
    assert!(
        line.contains("]    Run "),
        "tool-call log lines must be indented three spaces after who tag; got {line:?}"
    );
}

#[tokio::test]
async fn h23_start_and_done_tool_summary_omit_payload_brackets() {
    let path = "src/acp/trace_line_write.rs";
    let (start_line, done_line) =
        super::kpop_stdout_logger_plan_check_bracket::tee_read_tool_bracket_pair_stdout(path).await;
    super::kpop_stdout_logger_plan_check_bracket::assert_payload_omits_brackets_after_who_tag(
        &start_line, &done_line,
    );
    let start_plain = crate::ansi_strip::strip_ansi_escapes(&start_line);
    let done_plain = crate::ansi_strip::strip_ansi_escapes(&done_line);
    super::kpop_stdout_logger_plan_check_bracket::assert_styled_tool_summary_payloads_match(
        start_plain.split(']').nth(1).expect("payload").trim(),
        done_plain.split(']').nth(1).expect("payload").trim(),
    );
}

#[tokio::test]
async fn h21_unstyled_tool_summary_omits_brackets() {
    let _guard = stdout_log_test_guard();
    let fixture = begin_stdout_log_fixture();
    let (mut writer, mut coalesce) = open_trace_writer(&fixture.trace_path).await;
    tee_coalesced_update(&mut writer, &mut coalesce, &execute_tool_done_json("tool_plain")).await;
    drop(writer);
    let stdout = finish_stdout_log_fixture(fixture);
    let line = stdout.lines().find(|l| l.contains("Run ")).expect("tool summary");
    assert!(is_log_timestamp_token(line.split_whitespace().next().unwrap_or("")));
    assert!(!line.contains("[Run"), "got {line:?}");
    assert!(
        line.contains("]    Run "),
        "tool-call log lines must be indented three spaces after who tag; got {line:?}"
    );
}
