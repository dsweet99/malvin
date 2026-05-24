//! ACP reader integration tests for tool-summary formatting.

use crate::tool_summary::{ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines};
use serde_json::json;

#[test]
fn tool_summary_lines_parses_execute_tool_call_start() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_reader_smoke",
            "kind": "execute",
            "status": "pending",
            "rawInput": {"command": "echo hi"}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(lines.log.contains("[tool] start"));
    assert!(lines.log.contains("tool_reader_smoke"));
}
