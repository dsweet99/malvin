use super::{ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines};
use serde_json::json;

#[test]
fn search_done_truncated_appends_suffix() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_search_trunc",
            "kind": "search",
            "status": "pending",
            "rawInput": {"pattern": "needle"}
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_search_trunc",
            "kind": "search",
            "status": "completed",
            "rawOutput": {"totalMatches": 2, "truncated": true}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(stdout.contains("needle"));
    assert!(stdout.contains("(truncated)"));
}

#[test]
fn search_done_without_query_or_matches_uses_fallback_line() {
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_search_fallback",
            "kind": "search",
            "status": "completed"
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert_eq!(lines.stdout.as_deref(), Some("Search · matches"));
}

#[test]
fn search_done_with_match_count_and_no_query() {
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_search_count",
            "kind": "search",
            "status": "completed",
            "rawOutput": {"totalMatches": 3}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert_eq!(lines.stdout.as_deref(), Some("Search · 3 matches"));
}

#[test]
fn search_done_query_without_raw_output() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_search_q_only",
            "kind": "search",
            "status": "pending",
            "rawInput": {"query": "needle"}
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_search_q_only",
            "kind": "search",
            "status": "completed"
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert_eq!(lines.stdout.as_deref(), Some("Search needle · matches"));
}

#[test]
fn search_done_truncated_without_query() {
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_search_trunc_only",
            "kind": "search",
            "status": "completed",
            "rawOutput": {"totalMatches": 1, "truncated": true}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    assert_eq!(
        lines.stdout.as_deref(),
        Some("Search · 1 matches (truncated)")
    );
}
