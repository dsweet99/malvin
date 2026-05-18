// PATH, logs, child health, and run timing smoke tests.

use std::time::Duration;

use crate::child_health::ChildHealth;
use crate::run_timing::{RunTiming, TimingPhase, attach_new_run_timing, record_llm};

fn wall_bounded_timing() -> std::sync::Arc<std::sync::Mutex<RunTiming>> {
    let timing = RunTiming::new_arc();
    let mut g = timing.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.mark_wall_start(std::time::Instant::now());
    g.mark_wall_end(std::time::Instant::now());
    drop(g);
    timing
}

#[test]
fn smoke_support_paths() {
    let _ = crate::support_paths::DEFAULT_ACP_RPC_TIMEOUT_SECS;
    let _ = crate::support_paths::acp_rpc_timeout_secs_from_env();
    crate::support_paths::init_from_env();
    assert!(crate::support_paths::command_line().is_some());
    assert!(crate::support_paths::lookup_bin_on_path("no_such_bin_malvin_coverage_x").is_none());
    let _ = crate::support_paths::agent_or_cursor_agent_bin();
    let _ = crate::support_paths::require_kiss_for_malvin("plan");
    assert!(crate::support_paths::format_logs_dir(Path::new("/no_such_run_dir_malvin")).is_err());
}

#[test]
fn smoke_child_health() {
    assert_eq!(crate::child_health::sample_child_health(0).cpu_time_total, 0);
    let grace = crate::child_health::silence_grace_for_rpc_timeout(Duration::from_secs(600));
    assert!(grace.as_millis() >= 50);

    let sample_time = std::time::Instant::now();
    let t0 = ChildHealth {
        exists: true,
        zombie: false,
        state_hint: None,
        counters_trusted: true,
        cpu_time_total: 0,
        thread_count: Some(1),
        voluntary_ctxt: Some(0),
        sample_time,
    };
    let t1 = ChildHealth {
        exists: true,
        zombie: false,
        state_hint: None,
        counters_trusted: true,
        cpu_time_total: 1,
        thread_count: Some(1),
        voluntary_ctxt: Some(0),
        sample_time,
    };
    assert!(crate::child_health::health_indicates_progress(&t0, &t1));
}

#[tokio::test]
async fn smoke_child_health_evaluate_silence_dead_pid() {
    let outcome = crate::child_health::evaluate_after_acp_silence(
        u32::MAX,
        Duration::from_millis(1),
    )
    .await;
    assert_eq!(
        outcome,
        crate::child_health::SilenceHealthOutcome::ChildNotRunning
    );
}

#[test]
fn smoke_run_timing_json() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let timing = wall_bounded_timing();
    record_llm(
        Some(&timing),
        TimingPhase::Summary,
        Duration::from_micros(1),
    );
    crate::run_timing::record_backoff(Some(&timing), Duration::from_micros(1));
    crate::run_timing::finalize_run_timing_json_only(tmp.path(), &timing).expect("timing json");

    let mut slot: Option<std::sync::Arc<std::sync::Mutex<RunTiming>>> = None;
    let t2 = attach_new_run_timing(&mut slot);
    assert!(slot.is_some());
    crate::run_timing::finalize_run_timing_json_only(tmp.path(), &t2).expect("attach timing");

    let json = tmp.path().join(crate::run_timing::RUN_TIMING_JSON_FILE);
    assert!(json.is_file());
    assert!(!crate::run_timing::RUN_TIMING_SUMMARY_PREFIX.is_empty());
}
