use std::time::{Duration, Instant};

use super::{ReviewPairId, RunTiming, TimingPhase, record_backoff, record_llm, report};

#[test]
fn run_timing_json_phases_and_review_pair_id_mapping() {
    let mut r = RunTiming::default();
    r.mark_wall_start(Instant::now());
    r.mark_wall_end(Instant::now());
    r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(10));
    let phases = report::to_json_value(&r).get("phases_ms").unwrap().clone();
    for key in [
        "check_plan",
        "implement",
        "review_1_review",
        "review_2_review",
        "concerns",
        "learn",
        "summary",
    ] {
        assert!(phases.get(key).is_some(), "missing {key}");
    }
    assert_eq!(ReviewPairId::One.review_phase(), TimingPhase::Review1Review);
    assert_eq!(ReviewPairId::Two.review_phase(), TimingPhase::Review2Review);
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
fn check_plan_phase_accumulates_timing() {
    let mut r = RunTiming::default();
    r.mark_wall_start(Instant::now());
    r.add_llm_phase(TimingPhase::CheckPlan, Duration::from_millis(100));
    r.add_llm_phase(TimingPhase::CheckPlan, Duration::from_millis(50));
    r.mark_wall_end(Instant::now());
    assert_eq!(r.check_plan, Duration::from_millis(150));
    assert_eq!(r.llm_wait, Duration::from_millis(150));
    let json = report::to_json_value(&r);
    let phases = json.get("phases_ms").unwrap();
    assert_eq!(phases.get("check_plan").unwrap().as_u64().unwrap(), 150);
}
