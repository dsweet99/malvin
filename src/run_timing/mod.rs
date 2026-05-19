//! Wall-clock and phase-bucketed LLM wait timing for agent runs.
//! JSON is always written to [`RUN_TIMING_JSON_FILE`]; `code`/`kpop` also print [`RUN_TIMING_SUMMARY_PREFIX`].

mod report;

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const RUN_TIMING_JSON_FILE: &str = "run_timing.json";

pub const RUN_TIMING_SUMMARY_PREFIX: &str = "TIMING: ";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimingPhase {
    CheckPlan,
    Implement,
    ReviewFanout,
    ReviewWrite,
    Concerns,
    Learn,
    Summary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReviewPairId {
    Fanout,
}

impl ReviewPairId {
    #[must_use]
    pub const fn review_phase(self) -> TimingPhase {
        let Self::Fanout = self;
        TimingPhase::ReviewFanout
    }
}

#[derive(Debug, Clone)]
pub struct RunTiming {
    wall_start: Option<Instant>,
    wall_end: Option<Instant>,
    llm_wait: Duration,
    agent_retry_backoff: Duration,
    check_plan: Duration,
    implement: Duration,
    implement_display_name: &'static str,
    review_fanout: Duration,
    review_write: Duration,
    concerns: Duration,
    learn: Duration,
    summary: Duration,
}

impl Default for RunTiming {
    fn default() -> Self {
        Self {
            wall_start: None,
            wall_end: None,
            llm_wait: Duration::ZERO,
            agent_retry_backoff: Duration::ZERO,
            check_plan: Duration::ZERO,
            implement: Duration::ZERO,
            implement_display_name: "implement",
            review_fanout: Duration::ZERO,
            review_write: Duration::ZERO,
            concerns: Duration::ZERO,
            learn: Duration::ZERO,
            summary: Duration::ZERO,
        }
    }
}

impl RunTiming {
    #[must_use]
    pub fn new_arc() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::default()))
    }

    pub const fn mark_wall_start(&mut self, at: Instant) {
        self.wall_start = Some(at);
    }

    pub const fn mark_wall_end(&mut self, at: Instant) {
        self.wall_end = Some(at);
    }

    pub const fn add_llm_phase(&mut self, phase: TimingPhase, d: Duration) {
        self.llm_wait = self.llm_wait.saturating_add(d);
        match phase {
            TimingPhase::CheckPlan => self.check_plan = self.check_plan.saturating_add(d),
            TimingPhase::Implement => self.implement = self.implement.saturating_add(d),
            TimingPhase::ReviewFanout => {
                self.review_fanout = self.review_fanout.saturating_add(d);
            }
            TimingPhase::ReviewWrite => {
                self.review_write = self.review_write.saturating_add(d);
            }
            TimingPhase::Concerns => self.concerns = self.concerns.saturating_add(d),
            TimingPhase::Learn => self.learn = self.learn.saturating_add(d),
            TimingPhase::Summary => self.summary = self.summary.saturating_add(d),
        }
    }

    pub const fn add_agent_retry_backoff(&mut self, d: Duration) {
        self.agent_retry_backoff = self.agent_retry_backoff.saturating_add(d);
    }

    pub const fn set_implement_display_name(&mut self, label: &'static str) {
        self.implement_display_name = label;
    }

    pub(crate) fn wall_duration(&self) -> Option<Duration> {
        match (self.wall_start, self.wall_end) {
            (Some(a), Some(b)) => Some(b.saturating_duration_since(a)),
            _ => None,
        }
    }

    #[must_use]
    pub fn elapsed_so_far(&self) -> Duration {
        self.wall_start.map_or(Duration::ZERO, |start| {
            Instant::now().saturating_duration_since(start)
        })
    }

    /// Writes timing JSON and prints the human-readable summary line.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::Error`] when writing under `run_dir` fails.
    pub fn write_json_and_print_summary(&self, run_dir: &Path) -> std::io::Result<()> {
        report::write_json_and_print_summary(self, run_dir)
    }

    /// Writes timing JSON without printing a summary line.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::Error`] when writing under `run_dir` fails.
    pub fn write_json_only(&self, run_dir: &Path) -> std::io::Result<()> {
        report::write_json_only(self, run_dir)
    }
}

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

#[cfg(test)]
mod timing_tests;

pub mod acp_post_run;
