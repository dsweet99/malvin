use std::path::Path;
use std::sync::{Arc, Mutex};

use super::{
    RunTiming, TimingPhase, attach_kpop_engine_loop_run_timing, finalize_and_emit_run_timing,
    record_llm, RUN_TIMING_JSON_FILE,
};
use std::time::Duration;

fn simulate_kpop_engine_accumulate_iteration(
    client: &mut crate::acp::AgentClient,
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
    llm_ms: u64,
) -> Result<(), String> {
    use crate::acp_post_run::{RunTimingAfterAcp, RunTimingSessionEnd, emit_run_timing_after_acp};

    record_llm(
        Some(timing),
        TimingPhase::Implement,
        Duration::from_millis(llm_ms),
    );
    emit_run_timing_after_acp(RunTimingAfterAcp {
        client,
        run_dir,
        timing,
        acp_result: Ok(()),
        session_end: RunTimingSessionEnd::AccumulateRun,
    })
}

fn read_run_timing_json(run_dir: &Path) -> serde_json::Value {
    serde_json::from_str(
        &std::fs::read_to_string(run_dir.join(RUN_TIMING_JSON_FILE)).expect("run_timing.json"),
    )
    .expect("json")
}

fn kpop_engine_loop_fixture() -> (tempfile::TempDir, std::path::PathBuf, Arc<Mutex<RunTiming>>) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let run_timing = attach_kpop_engine_loop_run_timing();
    (tmp, run_dir, run_timing)
}

#[test]
fn kpop_engine_loop_timing_reports_wall_clock_after_finalize() {
    let (_tmp, run_dir, run_timing) = kpop_engine_loop_fixture();
    let mut client = crate::test_agent_client::smoke_agent_client();
    client.set_run_timing(Some(Arc::clone(&run_timing)));
    simulate_kpop_engine_accumulate_iteration(&mut client, &run_dir, &run_timing, 100)
        .expect("first gate-kpop iteration");
    simulate_kpop_engine_accumulate_iteration(&mut client, &run_dir, &run_timing, 200)
        .expect("second gate-kpop iteration");
    finalize_and_emit_run_timing(&run_dir, &run_timing).expect("finalize at run end");
    let json = read_run_timing_json(&run_dir);
    assert_eq!(json["llm_wait_ms"].as_u64(), Some(300));
    assert!(
        json["wall_clock_ms"].as_u64().is_some(),
        "gate-kpop code run must report wall_clock_ms (not wall = n/a); json={json}"
    );
}

#[test]
fn kpop_engine_loop_default_timing_without_wall_start_regression() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run_dir = tmp.path().join("run");
    std::fs::create_dir_all(&run_dir).expect("mkdir");
    let timing = RunTiming::new_arc();
    record_llm(
        Some(&timing),
        TimingPhase::Implement,
        Duration::from_millis(100),
    );
    finalize_and_emit_run_timing(&run_dir, &timing).expect("finalize");
    let json = read_run_timing_json(&run_dir);
    assert!(
        json["wall_clock_ms"].is_null(),
        "RunTiming::default() without mark_wall_start must not invent wall_clock_ms"
    );
}

#[test]
fn kpop_engine_accumulate_run_timing_sums_llm_wait_across_iterations() {
    let mut client = crate::test_agent_client::smoke_agent_client();
    let (_tmp, run_dir, _) = kpop_engine_loop_fixture();
    let timing = client.ensure_run_timing_for_session();
    simulate_kpop_engine_accumulate_iteration(&mut client, &run_dir, &timing, 900_000)
        .expect("first gate-kpop iteration");
    let timing_second = client.ensure_run_timing_for_session();
    assert!(Arc::ptr_eq(&timing, &timing_second));
    simulate_kpop_engine_accumulate_iteration(&mut client, &run_dir, &timing_second, 2_000)
        .expect("second gate-kpop iteration");
    finalize_and_emit_run_timing(&run_dir, &timing_second).expect("finalize at run end");
    let json = read_run_timing_json(&run_dir);
    assert_eq!(json["llm_wait_ms"].as_u64(), Some(902_000));
    assert!(client.timing.is_some());
}
