//! Wall-clock and phase-bucketed LLM wait timing for `malvin code` and `malvin kpop` runs.
//!
//! **Streams:** Summary uses `println!` (stdout); JSON is written under the run directory — see root `grounding.md`.

mod report;

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// JSON artifact filename under [`crate::artifacts::RunArtifacts::run_dir`].
pub const RUN_TIMING_JSON_FILE: &str = "run_timing.json";

/// One line printed to stdout after the workflow body (before the stderr post-run hint; `malvin code` / `malvin kpop`).
pub const RUN_TIMING_SUMMARY_PREFIX: &str = "Run timing:";

/// Which `session/prompt` turn to attribute LLM wait to (cumulative per label).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimingPhase {
    Implement,
    Review1Review,
    Review1Kpop,
    Review2Review,
    Review2Kpop,
    Concerns,
    Learn,
}

/// Review phase (`Review-1` vs `Review-2` in orchestrator progress labels).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReviewPairId {
    One,
    Two,
}

impl ReviewPairId {
    #[must_use]
    pub const fn review_phase(self) -> TimingPhase {
        match self {
            Self::One => TimingPhase::Review1Review,
            Self::Two => TimingPhase::Review2Review,
        }
    }

    #[must_use]
    pub const fn kpop_phase(self) -> TimingPhase {
        match self {
            Self::One => TimingPhase::Review1Kpop,
            Self::Two => TimingPhase::Review2Kpop,
        }
    }
}

/// Mutable accumulator; wall clock is bounded by orchestrator (`Instant` monotonic).
#[derive(Debug, Clone)]
pub struct RunTiming {
    wall_start: Option<Instant>,
    wall_end: Option<Instant>,
    llm_wait: Duration,
    agent_retry_backoff: Duration,
    implement: Duration,
    review_1_review: Duration,
    review_1_kpop: Duration,
    review_2_review: Duration,
    review_2_kpop: Duration,
    concerns: Duration,
    learn: Duration,
}

impl Default for RunTiming {
    fn default() -> Self {
        Self {
            wall_start: None,
            wall_end: None,
            llm_wait: Duration::ZERO,
            agent_retry_backoff: Duration::ZERO,
            implement: Duration::ZERO,
            review_1_review: Duration::ZERO,
            review_1_kpop: Duration::ZERO,
            review_2_review: Duration::ZERO,
            review_2_kpop: Duration::ZERO,
            concerns: Duration::ZERO,
            learn: Duration::ZERO,
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
            TimingPhase::Implement => self.implement = self.implement.saturating_add(d),
            TimingPhase::Review1Review => self.review_1_review = self.review_1_review.saturating_add(d),
            TimingPhase::Review1Kpop => self.review_1_kpop = self.review_1_kpop.saturating_add(d),
            TimingPhase::Review2Review => self.review_2_review = self.review_2_review.saturating_add(d),
            TimingPhase::Review2Kpop => self.review_2_kpop = self.review_2_kpop.saturating_add(d),
            TimingPhase::Concerns => self.concerns = self.concerns.saturating_add(d),
            TimingPhase::Learn => self.learn = self.learn.saturating_add(d),
        }
    }

    pub const fn add_agent_retry_backoff(&mut self, d: Duration) {
        self.agent_retry_backoff = self.agent_retry_backoff.saturating_add(d);
    }

    pub(crate) fn wall_duration(&self) -> Option<Duration> {
        match (self.wall_start, self.wall_end) {
            (Some(a), Some(b)) => Some(b.saturating_duration_since(a)),
            _ => None,
        }
    }

    /// Writes `run_timing.json` and prints one stdout summary line (timestamp-prefixed).
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON file cannot be created or written.
    pub fn write_json_and_print_summary(&self, run_dir: &Path) -> std::io::Result<()> {
        report::write_json_and_print_summary(self, run_dir)
    }
}

/// Installs a fresh [`RunTiming`] in `timing_slot` and records wall-clock start at [`Instant::now`].
///
/// `malvin code` and `malvin kpop` both use this so attachment stays consistent.
#[must_use]
pub fn attach_new_run_timing(timing_slot: &mut Option<Arc<Mutex<RunTiming>>>) -> Arc<Mutex<RunTiming>> {
    let timing = RunTiming::new_arc();
    *timing_slot = Some(Arc::clone(&timing));
    timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .mark_wall_start(Instant::now());
    timing
}

/// Records one LLM wait interval into `timing`, if present.
pub fn record_llm(
    timing: Option<&Arc<Mutex<RunTiming>>>,
    phase: TimingPhase,
    elapsed: Duration,
) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    g.add_llm_phase(phase, elapsed);
}

/// Records bounded-retry sleep duration (not model time).
pub fn record_backoff(timing: Option<&Arc<Mutex<RunTiming>>>, d: Duration) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    g.add_agent_retry_backoff(d);
}

/// Finalizes wall clock if needed, writes JSON, prints the stdout summary line.
///
/// # Errors
///
/// Returns an error if writing `run_timing.json` fails (see [`RunTiming::write_json_and_print_summary`]).
pub fn finalize_and_emit_run_timing(run_dir: &Path, timing: &Arc<Mutex<RunTiming>>) -> std::io::Result<()> {
    let mut g = timing.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if g.wall_end.is_none() {
        g.mark_wall_end(Instant::now());
    }
    let snapshot = g.clone();
    drop(g);
    snapshot.write_json_and_print_summary(run_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_timing_json_includes_phase_keys() {
        let mut r = RunTiming::default();
        r.mark_wall_start(Instant::now());
        r.mark_wall_end(Instant::now());
        r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(10));
        let v = report::to_json_value(&r);
        let phases = v.get("phases_ms").unwrap();
        for key in [
            "implement",
            "review_1_review",
            "review_1_kpop",
            "review_2_review",
            "review_2_kpop",
            "concerns",
            "learn",
        ] {
            assert!(phases.get(key).is_some(), "missing {key}");
        }
    }

    #[test]
    fn review_pair_id_maps_phases() {
        assert_eq!(ReviewPairId::One.review_phase(), TimingPhase::Review1Review);
        assert_eq!(ReviewPairId::Two.kpop_phase(), TimingPhase::Review2Kpop);
    }
}
