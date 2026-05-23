use crate::tool_summary::{ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines};
use serde_json::json;

#[test]
fn stdout_execute_completed_stderr_without_exit_code_must_not_show_checkmark() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_completed_stderr",
            "kind": "execute",
            "status": "pending",
            "rawInput": {"command": "cargo build"}
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_completed_stderr",
            "kind": "execute",
            "status": "completed",
            "rawOutput": {"stdout": "", "stderr": "error: build failed"}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.contains('✗'),
        "completed execute with stderr but no exitCode must not read as success; got {stdout:?}"
    );
    assert!(
        !stdout.contains('✓'),
        "completed execute with stderr but no exitCode must not show checkmark; got {stdout:?}"
    );
}

#[test]
fn stdout_execute_failed_without_exit_code_must_not_show_checkmark() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_fail_no_code",
            "kind": "execute",
            "status": "pending",
            "rawInput": {"command": "cargo build"}
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_fail_no_code",
            "kind": "execute",
            "status": "failed",
            "rawOutput": {"stdout": "", "stderr": "error: build failed"}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.contains('✗'),
        "failed execute without exitCode must not read as success; got {stdout:?}"
    );
    assert!(
        !stdout.contains('✓'),
        "failed execute without exitCode must not show checkmark; got {stdout:?}"
    );
}

#[test]
fn stdout_read_done_without_raw_output_still_emits_prose() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_read_no_raw",
            "kind": "read",
            "status": "pending",
            "title": "Read File"
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_read_no_raw",
            "kind": "read",
            "status": "completed"
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert!(
        lines.stdout.is_some(),
        "read done without rawOutput must still tee human summary when log has [tool] done"
    );
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.starts_with("Read "),
        "expected read done prose on stdout; got {stdout:?}"
    );
}

#[test]
fn stdout_search_done_without_raw_output_still_emits_prose() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_search_no_raw",
            "kind": "search",
            "status": "pending",
            "title": "grep"
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_search_no_raw",
            "kind": "search",
            "status": "completed"
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert!(
        lines.stdout.is_some(),
        "search done without rawOutput must still tee human summary when log has [tool] done"
    );
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.starts_with("Search "),
        "expected search done prose on stdout; got {stdout:?}"
    );
}

#[test]
fn stdout_search_done_empty_raw_output_still_emits_prose() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_search_empty_raw",
            "kind": "search",
            "status": "pending",
            "title": "grep"
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_search_empty_raw",
            "kind": "search",
            "status": "completed",
            "rawOutput": {}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert!(
        lines.stdout.is_some(),
        "search done with empty rawOutput must still tee human summary when log has [tool] done"
    );
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.starts_with("Search "),
        "expected search done prose on stdout; got {stdout:?}"
    );
}

#[test]
fn stdout_pending_update_suppresses_generic_read_subject() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_pend_out",
            "kind": "read",
            "status": "pending",
            "title": "Read File"
        }}
    });
    let pending = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_pend_out",
            "kind": "read",
            "status": "pending"
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&pending, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert!(
        lines.stdout.is_none(),
        "pending generic read should be suppressed until path details exist; got {:?}",
        lines.stdout
    );
}

#[test]
fn stdout_pending_update_uses_start_path_and_line_range_when_available() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_pend_path",
            "kind": "read",
            "status": "pending",
            "rawInput": {"path": "src/acp/tool_summary.rs", "startLine": 42, "endLine": 50}
        }}
    });
    let pending = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_pend_path",
            "kind": "read",
            "status": "pending"
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&pending, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.contains("Reading src/acp/tool_summary.rs:42-50"),
        "pending should surface persisted path/range details; got {stdout:?}"
    );
}
