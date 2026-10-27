use std::time::{Duration, Instant};

use super::*;
use crate::run_timing::process_ledger::{Ledger, absorb};

fn seed_timing(timing: &Arc<Mutex<RunTiming>>, closed: bool) -> Instant {
    let start = Instant::now() - Duration::from_secs(10);
    let mut g = timing.lock().unwrap();
    g.mark_wall_start(start);
    if closed {
        g.mark_wall_end(start + Duration::from_secs(2));
    }
    g.add_llm_phase(TimingPhase::Implement, Duration::from_millis(100));
    g.add_agent_retry_backoff(Duration::from_millis(30));
    for kind in ["read", "search", "edit", "execute", "other"] {
        g.add_tool_call_wall(kind, Duration::from_millis(5));
    }
    g.note_acp_tool_call_start();
    start
}

fn check_carried_timing(timing: &RunTiming, start: Instant) {
    assert_eq!(timing.wall_start, Some(start));
    assert!(timing.wall_end.is_none());
    let json = report::to_json_value(timing);
    assert_eq!(json["llm_wait_ms"], 100);
    assert_eq!(json["phases_ms"]["implement"], 100);
    assert_eq!(json["agent_retry_backoff_ms"], 30);
    assert_eq!(json["tool_calls_ms"], 25);
    for kind in ["read", "search", "edit", "execute", "other"] {
        assert_eq!(json["tool_calls_by_type_ms"][kind], 5);
    }
    assert_eq!(json["tokens"]["tool_call_starts"], 1);
}

fn replacement_preserves_timing(closed: bool) {
    let tmp = tempfile::tempdir().unwrap();
    let mut slot = None;
    let first = attach_new_run_timing(&mut slot, "cursor:first");
    let start = seed_timing(&first, closed);
    let mut totals = Ledger::default();
    absorb(&mut first.lock().unwrap(), &mut totals);
    for model in ["cursor:backup", "cursor:last"] {
        let next = attach_new_run_timing(&mut slot, model);
        let mut g = next.lock().unwrap();
        check_carried_timing(&g, start);
        g.note_acp_tool_call_completion();
        assert_eq!(
            g.steps, 0,
            "a new session cannot complete a prior tool batch"
        );
        absorb(&mut g, &mut totals);
    }
    let last = slot.unwrap();
    record_llm(
        Some(&last),
        TimingPhase::Implement,
        Duration::from_millis(40),
    );
    absorb(&mut last.lock().unwrap(), &mut totals);
    assert_eq!(totals.llm_wait, Duration::from_millis(140));
    assert_eq!(totals.tool_calls, Duration::from_millis(25));
    finalize_run_timing_json_only(tmp.path(), &last).unwrap();
    let json: serde_json::Value = serde_json::from_slice(
        &std::fs::read(tmp.path().join(crate::run_timing::RUN_TIMING_JSON_FILE)).unwrap(),
    )
    .unwrap();
    assert!(json["wall_clock_ms"].as_u64().unwrap() >= 10_000);
    assert_eq!(json["llm_wait_ms"], 140);
}

#[test]
fn session_replacement_keeps_open_and_closed_timing_without_recounting() {
    replacement_preserves_timing(false);
    replacement_preserves_timing(true);
}
