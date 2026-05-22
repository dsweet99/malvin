use crate::acp::tool_summary::{ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines};
use serde_json::json;

#[test]
fn edit_done_includes_path_and_counts() {
    let path = format!(
        "src/{}/nested/{}/deep/{}/module/file.rs",
        "segment".repeat(8),
        "part".repeat(8),
        "dir".repeat(8)
    );
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_e",
            "kind": "edit",
            "status": "completed",
            "rawOutput": {
                "path": path,
                "lineNumber": 12,
                "linesAdded": 280,
                "linesRemoved": 0
            }
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(lines.log.contains("path="));
    assert!(lines.log.contains(":12"));
    assert!(lines.log.contains("added=280"));
    assert!(lines.log.contains("removed=0"));
    assert!(lines.log.contains("files=1"));
    assert!(lines.log.contains("..."));
}

#[test]
fn read_done_uses_content_length() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_r",
            "kind": "read",
            "status": "completed",
            "rawOutput": {"content": "hello"}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(lines.log.contains("output=5B"));
}

#[test]
fn execute_done_error_headline_from_stderr() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_err",
            "kind": "execute",
            "status": "failed",
            "rawOutput": {
                "exitCode": 1,
                "stdout": "",
                "stderr": "error: build failed"
            }
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(lines.log.contains("error=\"error: build failed\""));
}

#[test]
fn search_done_matches_and_truncated() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_s",
            "kind": "search",
            "status": "completed",
            "rawOutput": {"totalMatches": 242, "truncated": true}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert!(lines.stdout.contains("matches=242"));
    assert!(lines.stdout.contains("truncated=true"));
}
