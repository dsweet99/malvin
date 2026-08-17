use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::{report, CostPolicy, RunTiming, TimingPhase};

fn token_cost_rates_from_home_config(model: &str) -> crate::malvin_config_file::TokenCostRates {
    crate::malvin_config_file::load_malvin_config(std::path::Path::new("."))
        .token_cost_rates_for(model)
}

#[must_use]
pub fn attach_new_run_timing(
    timing_slot: &mut Option<Arc<Mutex<RunTiming>>>,
    model: &str,
) -> Arc<Mutex<RunTiming>> {
    attach_new_run_timing_with_cost_policy(timing_slot, super::cost_policy_for_model(model), model)
}

#[must_use]
pub fn attach_new_run_timing_with_cost_policy(
    timing_slot: &mut Option<Arc<Mutex<RunTiming>>>,
    cost_policy: CostPolicy,
    model: &str,
) -> Arc<Mutex<RunTiming>> {
    let timing = RunTiming::new_arc();
    *timing_slot = Some(Arc::clone(&timing));
    {
        let mut g = timing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        g.mark_wall_start(Instant::now());
        g.token_cost_rates = token_cost_rates_from_home_config(model);
        g.cost_policy = cost_policy;
    }
    timing
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

pub fn finalize_and_emit_run_timing(
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
) -> std::io::Result<()> {
    finalize_snapshot(timing).write_json_and_print_summary(run_dir)
}

pub fn finalize_run_timing_json_only(
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
) -> std::io::Result<()> {
    finalize_snapshot(timing).write_json_only(run_dir)
}

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
