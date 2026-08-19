use super::{ToolSummaryDetail, ToolSummaryTracker, tool_summary_lines};
use serde_json::json;

fn tool_call_json(id: &str, session_update: &str, status: &str, kind: &str) -> serde_json::Value {
    json!({
        "method": "session/update",
        "params": {"update": {
            "sessionUpdate": session_update,
            "toolCallId": id,
            "kind": kind,
            "status": status,
            "title": kind
        }}
    })
}

fn timing_json_from(
    timing: &std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
) -> serde_json::Value {
    let tmp = tempfile::tempdir().unwrap();
    timing.lock().unwrap().write_json_only(tmp.path()).unwrap();
    let path = tmp.path().join(crate::run_timing::RUN_TIMING_JSON_FILE);
    serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
}

fn tool_calls_ms_from_timing(
    timing: &std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
) -> u64 {
    timing_json_from(timing)
        .get("tool_calls_ms")
        .and_then(serde_json::Value::as_u64)
        .unwrap()
}

fn tool_calls_by_type_ms(
    timing: &std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
    key: &str,
) -> u64 {
    timing_json_from(timing)
        .get("tool_calls_by_type_ms")
        .and_then(|v| v.get(key))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0)
}

fn complete_timed_tool(tracker: &mut ToolSummaryTracker, id: &str, kind: &str, sleep_ms: u64) {
    let start = tool_call_json(id, "tool_call", "pending", kind);
    let done = tool_call_json(id, "tool_call_update", "completed", kind);
    tool_summary_lines(&start, tracker, ToolSummaryDetail::Log).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(sleep_ms));
    tool_summary_lines(&done, tracker, ToolSummaryDetail::Log).unwrap();
}

fn tracker_with_timing() -> (
    ToolSummaryTracker,
    std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
) {
    let timing = crate::run_timing::RunTiming::new_arc();
    let mut tracker = ToolSummaryTracker::default();
    tracker.set_run_timing(Some(std::sync::Arc::clone(&timing)));
    (tracker, timing)
}

#[test]
fn completed_tool_call_evicts_tracker_when_run_timing_wired() {
    let (mut tracker, _timing) = tracker_with_timing();
    complete_timed_tool(&mut tracker, "tool_done", "read", 0);
    assert_eq!(tracker.call_count(), 0);
}

#[test]
fn completed_tool_call_records_wall_time_when_run_timing_wired() {
    let (mut tracker, timing) = tracker_with_timing();
    complete_timed_tool(&mut tracker, "tool_timed", "read", 15);
    assert_eq!(tracker.call_count(), 0);
    assert!(
        tool_calls_ms_from_timing(&timing) >= 10,
        "expected at least 10ms of tool wall time, got {}",
        tool_calls_ms_from_timing(&timing)
    );
    assert!(
        tool_calls_by_type_ms(&timing, "read") >= 10,
        "read bucket should receive the duration"
    );
}

#[test]
fn distinct_tool_kinds_accumulate_in_independent_timing_buckets() {
    let (mut tracker, timing) = tracker_with_timing();
    complete_timed_tool(&mut tracker, "tool_read", "read", 12);
    complete_timed_tool(&mut tracker, "tool_exec", "execute", 12);
    let read_ms = tool_calls_by_type_ms(&timing, "read");
    let exec_ms = tool_calls_by_type_ms(&timing, "execute");
    assert!(read_ms >= 10, "read bucket ms={read_ms}");
    assert!(exec_ms >= 10, "execute bucket ms={exec_ms}");
    assert_eq!(tool_calls_by_type_ms(&timing, "search"), 0);
    assert_eq!(tool_calls_by_type_ms(&timing, "edit"), 0);
    assert!(tool_calls_ms_from_timing(&timing) >= read_ms + exec_ms);
}

#[test]
fn incomplete_tool_call_does_not_record_wall_time() {
    let (mut tracker, timing) = tracker_with_timing();
    let start = tool_call_json("tool_pending", "tool_call", "pending", "read");
    tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    assert_eq!(tool_calls_ms_from_timing(&timing), 0);
}

#[test]
fn completed_tool_call_without_run_timing_does_not_accumulate() {
    let timing = crate::run_timing::RunTiming::new_arc();
    let mut tracker_no_timing = ToolSummaryTracker::default();
    let done = tool_call_json("tool_orphan", "tool_call_update", "completed", "read");
    tool_summary_lines(&done, &mut tracker_no_timing, ToolSummaryDetail::Log).unwrap();
    assert_eq!(tool_calls_ms_from_timing(&timing), 0);
}

#[test]
fn parallel_tool_call_starts_count_as_one_agent_step() {
    let (mut tracker, timing) = tracker_with_timing();
    for id in ["a", "b", "c"] {
        let start = tool_call_json(id, "tool_call", "pending", "read");
        tool_summary_lines(&start, &mut tracker, ToolSummaryDetail::Log).unwrap();
    }
    for id in ["a", "b", "c"] {
        let done = tool_call_json(id, "tool_call_update", "completed", "read");
        tool_summary_lines(&done, &mut tracker, ToolSummaryDetail::Log).unwrap();
    }
    let v = timing_json_from(&timing);
    assert_eq!(v["tokens"]["steps"], 1);
    assert_eq!(v["tokens"]["tool_call_starts"], 3);
}

#[test]
fn sequential_tool_batches_count_separate_agent_steps() {
    let (mut tracker, timing) = tracker_with_timing();
    complete_timed_tool(&mut tracker, "t1", "read", 0);
    complete_timed_tool(&mut tracker, "t2", "execute", 0);
    let v = timing_json_from(&timing);
    assert_eq!(v["tokens"]["steps"], 2);
}
