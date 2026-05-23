use crate::tool_summary::{
    ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines, tool_summary_stdout_display,
};
use crate::ansi_strip::strip_ansi_escapes;
use serde_json::json;

#[test]
fn stdout_read_done_prose_and_humanized_size() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_h_read",
            "kind": "read",
            "status": "completed",
            "rawOutput": {"content": "x".repeat(16_232)}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(stdout.starts_with("Read "));
    assert!(stdout.contains("16 KB"));
    assert!(!stdout.contains("[tool]"));
    assert!(!stdout.contains("id="));
}

#[test]
fn stdout_start_suppressed_until_running_threshold() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_h_run",
            "kind": "execute",
            "status": "pending",
            "rawInput": {"command": "sleep 2"}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let s = tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert!(s.stdout.is_none());
}

#[test]
fn stdout_execute_failure_shows_exit_and_error() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_h_fail",
            "kind": "execute",
            "status": "failed",
            "rawOutput": {
                "exitCode": 101,
                "stdout": "",
                "stderr": "error: build failed"
            }
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(stdout.contains("✗ exit 101"));
    assert!(stdout.contains("build failed"));
}

#[test]
fn stdout_display_ansi_stripped_matches_plain() {
    let plain = "Run cargo test · 1.0s · ✓";
    let display = tool_summary_stdout_display(plain);
    assert_eq!(strip_ansi_escapes(&display), plain);
}

#[test]
fn log_channel_stays_key_value() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_h_log",
            "kind": "search",
            "status": "completed",
            "rawOutput": {"totalMatches": 3, "truncated": false}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(lines.log.contains("[tool] done"));
    assert!(lines.log.contains("matches=3"));
    assert!(lines.log.contains("truncated=false"));
}
