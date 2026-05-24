use super::{ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines};
use serde_json::json;

fn tool_call_json(id: &str, session_update: &str, status: &str) -> serde_json::Value {
    json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": session_update,
            "toolCallId": id,
            "kind": "read",
            "status": status,
            "title": "Read"
        }}
    })
}

fn tool_calls_ms_from_timing(
    timing: &std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
) -> u64 {
    let tmp = tempfile::tempdir().unwrap();
    timing
        .lock()
        .unwrap()
        .write_json_only(tmp.path())
        .unwrap();
    let path = tmp.path().join(crate::run_timing::RUN_TIMING_JSON_FILE);
    let json: serde_json::Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    json.get("tool_calls_ms")
        .and_then(serde_json::Value::as_u64)
        .unwrap()
}

#[test]
fn completed_tool_call_evicts_tracker_when_run_timing_wired() {
    let timing = crate::run_timing::RunTiming::new_arc();
    let mut tracker = ToolSummaryTracker::default();
    tracker.set_run_timing(Some(std::sync::Arc::clone(&timing)));

    let start = tool_call_json("tool_done", "tool_call", "pending");
    let done = tool_call_json("tool_done", "tool_call_update", "completed");
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Log).unwrap();

    assert_eq!(tracker.call_count(), 0);
    assert_eq!(tool_calls_ms_from_timing(&timing), 0);
}

#[test]
fn incomplete_tool_call_does_not_record_wall_time() {
    let timing = crate::run_timing::RunTiming::new_arc();
    let mut tracker = ToolSummaryTracker::default();
    tracker.set_run_timing(Some(std::sync::Arc::clone(&timing)));

    let start = tool_call_json("tool_pending", "tool_call", "pending");
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();

    assert_eq!(tool_calls_ms_from_timing(&timing), 0);
}

#[test]
fn completed_tool_call_without_run_timing_does_not_accumulate() {
    let timing = crate::run_timing::RunTiming::new_arc();
    let mut tracker_no_timing = ToolSummaryTracker::default();
    let done = tool_call_json("tool_orphan", "tool_call_update", "completed");
    tool_summary_lines(&done, &mut tracker_no_timing, ToolSummaryDetail::Log).unwrap();

    assert_eq!(tool_calls_ms_from_timing(&timing), 0);
}
