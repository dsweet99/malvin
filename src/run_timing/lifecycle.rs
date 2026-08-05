use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::{report, RunTiming, TimingPhase};

#[must_use]
pub fn attach_new_run_timing(
    timing_slot: &mut Option<Arc<Mutex<RunTiming>>>,
) -> Arc<Mutex<RunTiming>> {
    let timing = RunTiming::new_arc();
    *timing_slot = Some(Arc::clone(&timing));
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .mark_wall_start(Instant::now());
    timing
}

/// Anchors one wall-clock interval for a gate-kpop `code` loop (shared across iterations).
#[must_use]
pub fn attach_kpop_engine_loop_run_timing() -> Arc<Mutex<RunTiming>> {
    let mut slot = None;
    attach_new_run_timing(&mut slot)
}

pub fn record_llm(timing: Option<&Arc<Mutex<RunTiming>>>, phase: TimingPhase, elapsed: Duration) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.add_llm_phase(phase, elapsed);
}

pub fn record_backoff(timing: Option<&Arc<Mutex<RunTiming>>>, d: Duration) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.add_agent_retry_backoff(d);
}

fn finalize_snapshot(timing: &Arc<Mutex<RunTiming>>) -> RunTiming {
    let mut g = timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if g.wall_end.is_none() {
        g.mark_wall_end(Instant::now());
    }
    g.finalize_acp_trailing_assistant_step();
    g.clone()
}

/// Finalizes wall clock end time and writes JSON plus the printed summary.
///
/// # Errors
///
/// Returns [`std::io::Error`] when writing under `run_dir` fails.
pub fn finalize_and_emit_run_timing(
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
) -> std::io::Result<()> {
    finalize_snapshot(timing).write_json_and_print_summary(run_dir)
}

/// Finalizes wall clock end time and writes JSON only.
///
/// # Errors
///
/// Returns [`std::io::Error`] when writing under `run_dir` fails.
pub fn finalize_run_timing_json_only(
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
) -> std::io::Result<()> {
    finalize_snapshot(timing).write_json_only(run_dir)
}

/// Persists in-progress timing without closing the run wall clock.
///
/// # Errors
///
/// Returns [`std::io::Error`] when writing under `run_dir` fails.
pub fn persist_open_run_timing_json(
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
) -> std::io::Result<()> {
    let snapshot = {
        let mut g = timing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.finalize_acp_trailing_assistant_step();
        g.clone()
    };
    report::write_json_only(&snapshot, run_dir)
}
