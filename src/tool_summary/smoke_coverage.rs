//! Behavioral and kiss per-file coverage for `tool_summary`.

use super::{
    ToolSummaryDetail, ToolSummaryTracker, shorten_middle, tool_summary_lines,
};
use serde_json::json;

#[test]
fn shorten_middle_preserves_ends() {
    let s = "src/very/long/path/to/some/deep/module/file.rs";
    let out = shorten_middle(s, 40);
    assert!(out.starts_with("src/very"));
    assert!(out.contains("..."));
    assert!(out.ends_with("file.rs"));
    assert!(out.chars().count() <= 40);
}

#[test]
fn shorten_middle_short_unchanged() {
    assert_eq!(shorten_middle("abc", 60), "abc");
}

#[test]
fn parse_tool_call_start() {
    let v = json!({
        "jsonrpc": "2.0",
        "method": "session/update",
        "params": {
            "update": {
                "sessionUpdate": "tool_call",
                "toolCallId": "tool_abc",
                "kind": "execute",
                "status": "pending",
                "title": "`cargo test`",
                "rawInput": {"command": "cargo test"}
            }
        }
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(lines.log.contains("[tool] start"));
    assert!(lines.log.contains("kind=execute"));
    assert!(lines.log.contains("tool_abc"));
    assert!(lines.log.contains("title=\"cargo test\""));
    assert!(!lines.log.contains("rawOutput"));
}

#[test]
fn parse_tool_update_running_and_done() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_x",
            "kind": "execute",
            "status": "pending",
            "rawInput": {"command": "echo hi"}
        }}
    });
    let running = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_x",
            "status": "in_progress"
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_x",
            "kind": "execute",
            "status": "completed",
            "rawOutput": {"exitCode": 0, "stdout": "ok\n", "stderr": ""}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let s = tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(s.log.contains("[tool] start"));
    assert!(
        s.log.contains("echo hi"),
        "start log should retain command: {}",
        s.log
    );
    let r = tool_summary_lines(&running, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(r.log.contains("[tool] running"));
    assert!(r.log.contains("elapsed="));
    assert_eq!(tracker.stored_command("tool_x"), Some("echo hi"));
    let d_out = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = d_out.stdout.as_deref().unwrap_or("");
    assert!(stdout.contains("Run echo hi"));
    assert!(stdout.contains('✓'));
    assert!(!stdout.contains("exit=0"));
    assert!(!stdout.contains("stdout="));
    let mut tracker_log = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker_log, ToolSummaryDetail::Log).unwrap();
    tool_summary_lines(&running, &mut tracker_log, ToolSummaryDetail::Log).unwrap();
    let d = tool_summary_lines(&done, &mut tracker_log, ToolSummaryDetail::Log).unwrap();
    assert!(d.log.contains("[tool] done"));
    assert!(d.log.contains("exit=0"));
    assert!(d.log.contains("stdout=3B"));
    assert!(!d.log.contains("ok\n"));
}

#[test]
fn done_summary_omits_title_field_per_plan() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_done_title",
            "kind": "execute",
            "status": "pending",
            "title": "`cargo nextest run`",
            "rawInput": {"command": "cargo nextest run"}
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_done_title",
            "kind": "execute",
            "status": "completed",
            "rawOutput": {"exitCode": 0, "stdout": "", "stderr": ""}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let start_line = tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(start_line.log.contains("title="));
    let done_line = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(
        done_line.log.contains("[tool] done"),
        "expected done summary, got {:?}",
        done_line.log
    );
    assert!(
        !done_line.log.contains("title="),
        "done lines must not repeat title= (plan done example has no title); got {:?}",
        done_line.log
    );
}

#[test]
fn edit_done_content_only_omits_synthetic_added_removed_counts() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_edit_content",
            "kind": "edit",
            "status": "completed",
            "rawOutput": {"content": "fn main() {\n}\n"}
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(
        !lines.log.contains("added=") && !lines.log.contains("removed="),
        "content-only edit completion must not invent added=/removed= counts; got {:?}",
        lines.log
    );
}

#[test]
fn edit_done_emits_added_when_only_lines_added_field_present() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_edit_partial",
            "kind": "edit",
            "status": "completed",
            "rawOutput": {
                "path": "src/foo.rs",
                "linesAdded": 280
            }
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(
        lines.log.contains("added=280"),
        "structured linesAdded alone must appear on edit done (plan added=280 removed=0); got {:?}",
        lines.log
    );
}

#[test]
fn edit_done_raw_output_uri_includes_path_in_log() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_edit_raw_uri",
            "kind": "edit",
            "status": "completed",
            "rawOutput": {
                "uri": "file:///workspace/review.md",
                "linesAdded": 1,
                "linesRemoved": 0
            }
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(
        lines.log.contains("path=") && lines.log.contains("review.md"),
        "log channel should include normalized uri path; log={}",
        lines.log
    );
}

#[test]
fn tool_call_update_pending_labeled_pending_not_start() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_pending",
            "kind": "execute",
            "status": "pending"
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert!(
        lines.log.contains("[tool] pending"),
        "tool_call_update with status=pending must not reuse [tool] start; got {:?}",
        lines.log
    );
    assert!(
        !lines.log.contains("[tool] start"),
        "pending update must be distinct from start; got {:?}",
        lines.log
    );
}

