//! Wall-clock and phase-bucketed LLM wait timing for `malvin code`, `malvin kpop`, and `malvin do` runs.
//!
//! **Streams:** One stdout line beginning with [`RUN_TIMING_SUMMARY_PREFIX`] (`TIMING: ` including the trailing space before the first `name = value` field) via [`crate::output::print_stdout_line`] (timestamp-prefixed `YYYYMMDD.HHMMSS.mmm:[malvin]: …`, same helper as other CLI stdout); the helper formats then prints the line. JSON is written under the run directory — see root `grounding.md`.

mod report;

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// JSON artifact filename under [`crate::artifacts::RunArtifacts::run_dir`].
pub const RUN_TIMING_JSON_FILE: &str = "run_timing.json";

/// One line printed to stdout after the workflow body (`malvin code` / `malvin kpop` / `malvin do`).
pub const RUN_TIMING_SUMMARY_PREFIX: &str = "TIMING: ";

/// Which `session/prompt` turn to attribute LLM wait to (cumulative per label).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimingPhase {
    CheckPlan,
    Implement,
    Review1Review,
    Review2Review,
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
}

/// Mutable accumulator; wall clock is bounded by orchestrator (`Instant` monotonic).
#[derive(Debug, Clone)]
pub struct RunTiming {
    wall_start: Option<Instant>,
    wall_end: Option<Instant>,
    llm_wait: Duration,
    agent_retry_backoff: Duration,
    check_plan: Duration,
    implement: Duration,
    implement_display_name: &'static str,
    review_1_review: Duration,
    review_2_review: Duration,
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
            check_plan: Duration::ZERO,
            implement: Duration::ZERO,
            implement_display_name: "implement",
            review_1_review: Duration::ZERO,
            review_2_review: Duration::ZERO,
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
            TimingPhase::CheckPlan => self.check_plan = self.check_plan.saturating_add(d),
            TimingPhase::Implement => self.implement = self.implement.saturating_add(d),
            TimingPhase::Review1Review => {
                self.review_1_review = self.review_1_review.saturating_add(d);
            }
            TimingPhase::Review2Review => {
                self.review_2_review = self.review_2_review.saturating_add(d);
            }
            TimingPhase::Concerns => self.concerns = self.concerns.saturating_add(d),
            TimingPhase::Learn => self.learn = self.learn.saturating_add(d),
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

    /// Returns elapsed time since wall start (for mid-run checks like conditional learn).
    #[must_use]
    pub fn elapsed_so_far(&self) -> Duration {
        self.wall_start
            .map_or(Duration::ZERO, |start| Instant::now().saturating_duration_since(start))
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
/// `malvin code`, `malvin kpop`, and `malvin do` use this via [`crate::acp::AgentClient::attach_run_timing_for_session`]
/// so attachment stays consistent.
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

/// Records one LLM wait interval into `timing`, if present.
pub fn record_llm(timing: Option<&Arc<Mutex<RunTiming>>>, phase: TimingPhase, elapsed: Duration) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.add_llm_phase(phase, elapsed);
}

/// Records bounded-retry sleep duration (not model time).
pub fn record_backoff(timing: Option<&Arc<Mutex<RunTiming>>>, d: Duration) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.add_agent_retry_backoff(d);
}

/// Finalizes wall clock if needed, writes JSON, prints the stdout summary line.
///
/// # Errors
///
/// Returns an error if writing `run_timing.json` fails (see [`RunTiming::write_json_and_print_summary`]).
pub fn finalize_and_emit_run_timing(
    run_dir: &Path,
    timing: &Arc<Mutex<RunTiming>>,
) -> std::io::Result<()> {
    let mut g = timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
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
    fn run_timing_json_phases_and_review_pair_id_mapping() {
        let mut r = RunTiming::default();
        r.mark_wall_start(Instant::now());
        r.mark_wall_end(Instant::now());
        r.add_llm_phase(TimingPhase::Implement, Duration::from_millis(10));
        let phases = report::to_json_value(&r).get("phases_ms").unwrap().clone();
        for key in ["check_plan", "implement", "review_1_review", "review_2_review", "concerns", "learn"] {
            assert!(phases.get(key).is_some(), "missing {key}");
        }
        assert_eq!(ReviewPairId::One.review_phase(), TimingPhase::Review1Review);
        assert_eq!(ReviewPairId::Two.review_phase(), TimingPhase::Review2Review);
    }

    #[test]
    fn elapsed_so_far_and_record_functions() {
        let mut r = RunTiming::default();
        assert_eq!(r.elapsed_so_far(), Duration::ZERO);
        r.mark_wall_start(Instant::now());
        std::thread::sleep(Duration::from_millis(10));
        assert!(r.elapsed_so_far() >= Duration::from_millis(5));
        let timing = RunTiming::new_arc();
        record_llm(Some(&timing), TimingPhase::Implement, Duration::from_millis(100));
        record_llm(Some(&timing), TimingPhase::Implement, Duration::from_millis(50));
        record_backoff(Some(&timing), Duration::from_millis(200));
        record_backoff(Some(&timing), Duration::from_millis(100));
        let g = timing.lock().unwrap();
        assert_eq!((g.implement, g.llm_wait, g.agent_retry_backoff),
            (Duration::from_millis(150), Duration::from_millis(150), Duration::from_millis(300)));
        drop(g);
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
}
