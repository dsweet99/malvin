use crate::tool_summary::{ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines};
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
    assert!(!lines.log.contains(":12"));
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
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(stdout.contains("242 matches"));
    assert!(stdout.contains("(truncated)"));
    assert!(!stdout.contains("truncated=false"));
}

#[test]
fn edit_done_content_diff_shows_filename_on_stdout() {
    let start = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call",
            "toolCallId": "tool_edit_content_diff",
            "kind": "edit",
            "status": "pending",
            "title": "Edit File",
            "rawInput": {}
        }}
    });
    let done = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_edit_content_diff",
            "status": "completed",
            "content": [{
                "type": "diff",
                "path": "review.md",
                "newText": "hello",
                "oldText": ""
            }]
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    let lines = tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.contains("Edit review.md"),
        "content[] diff path should appear in edit summary; got {stdout:?}"
    );
    assert!(
        !stdout.contains("Edit file"),
        "edit summary must not fall back to generic file; got {stdout:?}"
    );
    assert!(
        lines.log.contains("path=review.md"),
        "log channel should include diff path; log={}",
        lines.log
    );
}

#[test]
fn edit_done_multi_content_diff_shows_file_count() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_edit_multi_diff",
            "kind": "edit",
            "status": "completed",
            "content": [
                {"type": "diff", "path": "src/a.rs", "newText": "a", "oldText": ""},
                {"type": "diff", "path": "src/b.rs", "newText": "b", "oldText": ""}
            ]
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.contains("Edit 2 files"),
        "multi-file content[] diff should show file count; got {stdout:?}"
    );
}

#[test]
fn edit_done_raw_output_paths_still_supported() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_edit_paths",
            "kind": "edit",
            "status": "completed",
            "rawOutput": {
                "paths": ["src/one.rs", "src/two.rs"]
            }
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.contains("Edit 2 files"),
        "rawOutput.paths should still produce multi-file summary; got {stdout:?}"
    );
}

#[test]
fn edit_done_content_diff_uri_shows_filename_on_stdout() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_edit_uri",
            "kind": "edit",
            "status": "completed",
            "content": [{
                "type": "diff",
                "uri": "file:///workspace/review.md",
                "newText": "hello",
                "oldText": ""
            }]
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.contains("review.md"),
        "content[] diff uri should appear in edit summary; got {stdout:?}"
    );
    assert!(
        !stdout.contains("Edit file"),
        "edit summary must not fall back to generic file when uri is present; got {stdout:?}"
    );
}

#[test]
fn edit_done_content_diff_uri_host_form_normalizes_filesystem_path() {
    let v = json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool_edit_uri_host",
            "kind": "edit",
            "status": "completed",
            "content": [{
                "type": "diff",
                "uri": "file://localhost/workspace/review.md",
                "newText": "hello",
                "oldText": ""
            }]
        }}
    });
    let mut tracker = ToolSummaryTracker::default();
    let lines = tool_summary_lines(&v, &mut tracker, ToolSummaryDetail::Stdout).unwrap();
    let stdout = lines.stdout.as_deref().unwrap_or("");
    assert!(
        stdout.contains("/workspace/review.md"),
        "host-form file URI should normalize to filesystem path; got {stdout:?}"
    );
    assert!(
        !stdout.contains("localhost"),
        "host-form file URI should not keep host segment in subject; got {stdout:?}"
    );
}
