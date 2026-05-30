use std::sync::{Arc, Mutex};

use super::{
    RunTiming, TimingPhase, attach_new_run_timing, finalize_and_emit_run_timing,
    finalize_run_timing_json_only, record_backoff, record_llm, report,
};
use std::time::{Duration, Instant};

#[test]
fn run_timing_json_phases_include_only_implement() {
    let mut r = RunTiming::default();
    r.mark_wall_start(Instant::now());
    r.mark_wall_end(Instant::now());
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(10));
    let phases = report::to_json_value(&r).get("phases_ms").unwrap().clone();
    assert!(phases.get("implement").is_some());
    for dead in [
        "check_plan",
        "review_fanout",
        "review_write",
        "concerns",
        "summary",
    ] {
        assert!(
            phases.get(dead).is_none(),
            "dead phase key {dead} must not appear in JSON"
        );
    }
}

#[test]
fn elapsed_so_far_advances_after_wall_start() {
    let mut r = RunTiming::default();
    assert_eq!(r.elapsed_so_far(), Duration::ZERO);
    r.mark_wall_start(Instant::now());
    std::thread::sleep(Duration::from_millis(10));
    assert!(r.elapsed_so_far() >= Duration::from_millis(5));
}

#[test]
fn record_llm_and_backoff_accumulate_on_arc_timing() {
    let timing = RunTiming::new_arc();
    record_llm(
        Some(&timing),
        TimingPhase::Implement,
        Duration::from_millis(100),
    );
    record_llm(
        Some(&timing),
        TimingPhase::Implement,
        Duration::from_millis(50),
    );
    record_backoff(Some(&timing), Duration::from_millis(200));
    record_backoff(Some(&timing), Duration::from_millis(100));
    let (impl_d, llm_d, backoff_d) = {
        let g = timing.lock().unwrap();
        (g.implement, g.llm_wait, g.agent_retry_backoff)
    };
    assert_eq!(
        (impl_d, llm_d, backoff_d),
        (
            Duration::from_millis(150),
            Duration::from_millis(150),
            Duration::from_millis(300)
        )
    );
}

#[test]
fn record_llm_and_backoff_noop_when_timing_slot_none() {
    record_llm(None, TimingPhase::Implement, Duration::from_millis(100));
    record_backoff(None, Duration::from_millis(100));
}

#[test]
fn implement_phase_accumulates_timing() {
    let mut r = RunTiming::default();
    r.mark_wall_start(Instant::now());
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(100));
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(50));
    r.mark_wall_end(Instant::now());
    assert_eq!(r.implement, Duration::from_millis(150));
    assert_eq!(r.llm_wait, Duration::from_millis(150));
    let json = report::to_json_value(&r);
    let phases = json.get("phases_ms").unwrap();
    assert_eq!(phases.get("implement").unwrap().as_u64().unwrap(), 150);
}

#[test]
fn attach_new_run_timing_and_finalize_json_only() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut slot: Option<Arc<Mutex<RunTiming>>> = None;
    let timing = attach_new_run_timing(&mut slot);
    assert!(slot.is_some());
    finalize_run_timing_json_only(tmp.path(), &timing).expect("json only");
    assert!(tmp.path().join(super::RUN_TIMING_JSON_FILE).is_file());
}

#[test]
fn tool_call_wall_duration_accumulates_in_run_timing() {
    let mut r = RunTiming::default();
    r.add_tool_call_wall(Duration::from_millis(30));
    r.add_tool_call_wall(Duration::from_millis(20));
    assert_eq!(
        report::to_json_value(&r)
            .get("tool_calls_ms")
            .and_then(serde_json::Value::as_u64),
        Some(50)
    );
}

#[test]
fn wall_clock_ms_for_json_uses_elapsed_when_wall_end_open() {
    let mut r = RunTiming::default();
    r.mark_wall_start(Instant::now());
    std::thread::sleep(Duration::from_millis(15));
    let ms = report::wall_clock_ms_for_json(&r).expect("wall ms");
    assert!(ms >= 10, "open run should report elapsed wall ms, got {ms}");
    assert!(r.wall_duration().is_none());
}

#[test]
fn persist_open_run_timing_json_keeps_wall_end_unset() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut slot: Option<Arc<Mutex<RunTiming>>> = None;
    let timing = attach_new_run_timing(&mut slot);
    record_llm(
        Some(&timing),
        TimingPhase::Implement,
        Duration::from_millis(100),
    );
    super::persist_open_run_timing_json(tmp.path(), &timing).expect("persist open");
    assert!(timing.lock().unwrap().wall_end.is_none());
    let json = std::fs::read_to_string(tmp.path().join(super::RUN_TIMING_JSON_FILE)).unwrap();
    assert!(json.contains("\"implement\": 100"));
    assert!(json.contains("\"llm_wait_ms\": 100"));
}

#[test]
fn accumulate_run_timing_across_two_sessions() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut slot: Option<Arc<Mutex<RunTiming>>> = None;
    let timing = attach_new_run_timing(&mut slot);
    record_llm(Some(&timing), TimingPhase::Implement, Duration::from_millis(1_000));
    super::persist_open_run_timing_json(tmp.path(), &timing).expect("first persist");
    record_llm(Some(&timing), TimingPhase::Implement, Duration::from_millis(500));
    finalize_and_emit_run_timing(tmp.path(), &timing).expect("finalize");
    let llm_ms = report::to_json_value(&timing.lock().unwrap())["llm_wait_ms"]
        .as_u64()
        .unwrap();
    assert_eq!(llm_ms, 1_500);
}

#[test]
fn run_timing_without_wall_start_yields_null_wall_ms() {
    let mut r = RunTiming::default();
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(100));
    assert!(report::wall_clock_ms_for_json(&r).is_none());
}

#[test]
fn attach_new_run_timing_enables_wall_ms_after_finalize() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut slot: Option<Arc<Mutex<RunTiming>>> = None;
    let timing = attach_new_run_timing(&mut slot);
    record_llm(
        Some(&timing),
        TimingPhase::Implement,
        Duration::from_millis(100),
    );
    finalize_and_emit_run_timing(tmp.path(), &timing).expect("finalize");
    let json: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(tmp.path().join(super::RUN_TIMING_JSON_FILE)).expect("run_timing.json"),
    )
    .expect("json");
    assert!(json["wall_clock_ms"].as_u64().is_some());
}

#[test]
fn finalize_and_emit_run_timing_writes_summary() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let timing = RunTiming::new_arc();
    {
        let mut g = timing.lock().unwrap();
        g.mark_wall_start(Instant::now());
        g.mark_wall_end(Instant::now());
    }
    finalize_and_emit_run_timing(tmp.path(), &timing).expect("emit");
    assert!(tmp.path().join(super::RUN_TIMING_JSON_FILE).is_file());
}
