//! Wall-clock and phase-bucketed LLM wait timing for agent runs.
//!
//! JSON is always written to [`RUN_TIMING_JSON_FILE`]; `code`/`kpop`/`router` also print
//! [`RUN_TIMING_SUMMARY_PREFIX`] and a combined `COST:` footnote (tokens + cost fields).

mod cost;
mod lifecycle;
mod report;
#[path = "report_cost_line.rs"]
mod report_cost_line;
mod tokens;

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub const RUN_TIMING_JSON_FILE: &str = "run_timing.json";

pub const RUN_TIMING_SUMMARY_PREFIX: &str = "TIMING: ";

pub use report_cost_line::RUN_COST_SUMMARY_PREFIX;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimingPhase {
    Implement,
}

/// Wire keys for per-type tool-call wall durations (ACP kinds + `other`).
pub const TOOL_CALL_TYPE_MS_KEYS: [&str; 5] = ["read", "search", "edit", "execute", "other"];

/// ACP concurrent-batch step proxy state (see `COST:` / pier agent steps).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum AcpStepProxy {
    #[default]
    Idle,
    OpenBatch,
    TrailingAssistant,
}

/// How `COST` footnote USD fields are produced for this run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CostPolicy {
    /// `cursor:` / `pi:`: estimate from per-model `usd_per_microtoken_*` × token counts / 1e6 (0 when rates are unset).
    #[default]
    EstimateFromRates,
    /// Treat every completion as cost `0` (reserved; unused after local-backend removal).
    Zero,
}

/// Choose [`CostPolicy`] from a prefixed model id (`cursor:` / `pi:`).
#[must_use]
pub const fn cost_policy_for_model(_model: &str) -> CostPolicy {
    CostPolicy::EstimateFromRates
}

#[derive(Debug, Clone)]
pub struct RunTiming {
    wall_start: Option<Instant>,
    wall_end: Option<Instant>,
    llm_wait: Duration,
    agent_retry_backoff: Duration,
    implement: Duration,
    implement_display_name: &'static str,
    tool_calls: Duration,
    tool_calls_read: Duration,
    tool_calls_search: Duration,
    tool_calls_edit: Duration,
    tool_calls_execute: Duration,
    tool_calls_other: Duration,
    pub(crate) tx_costs: Vec<f64>,
    pub(crate) unknown_tx_count: u32,
    /// Cursor-mode rates for estimating USD cost from token usage.
    pub(crate) token_cost_rates: crate::malvin_config_file::TokenCostRates,
    /// Backend-specific cost filling policy (`cursor:` / `pi:`).
    pub(crate) cost_policy: CostPolicy,
    pub(crate) steps: u64,
    /// `None` until at least one input token count is observed.
    pub(crate) tokens_in: Option<u64>,
    /// `None` until at least one output token count is observed.
    pub(crate) tokens_out: Option<u64>,
    pub(crate) cache_read: Option<u64>,
    pub(crate) cache_write: Option<u64>,
    pub(crate) tool_call_starts: u64,
    pub(crate) usage_tx_count: u32,
    pub(crate) unknown_usage_tx_count: u32,
    pub(crate) acp_step_proxy: AcpStepProxy,
}

impl Default for RunTiming {
    fn default() -> Self {
        Self {
            wall_start: None,
            wall_end: None,
            llm_wait: Duration::ZERO,
            agent_retry_backoff: Duration::ZERO,
            implement: Duration::ZERO,
            implement_display_name: "implement",
            tool_calls: Duration::ZERO,
            tool_calls_read: Duration::ZERO,
            tool_calls_search: Duration::ZERO,
            tool_calls_edit: Duration::ZERO,
            tool_calls_execute: Duration::ZERO,
            tool_calls_other: Duration::ZERO,
            tx_costs: Vec::new(),
            unknown_tx_count: 0,
            token_cost_rates: crate::malvin_config_file::TokenCostRates::default(),
            cost_policy: CostPolicy::EstimateFromRates,
            steps: 0,
            tokens_in: None,
            tokens_out: None,
            cache_read: None,
            cache_write: None,
            tool_call_starts: 0,
            usage_tx_count: 0,
            unknown_usage_tx_count: 0,
            acp_step_proxy: AcpStepProxy::Idle,
        }
    }
}

impl RunTiming {
    /// Adds wall time for one completed tool call, attributed by ACP wire `kind`.
    /// Unknown kinds accumulate under `other`. Aggregate `tool_calls` is always updated.
    pub fn add_tool_call_wall(&mut self, kind: &str, d: Duration) {
        self.tool_calls = self.tool_calls.saturating_add(d);
        let bucket = match kind {
            "read" => &mut self.tool_calls_read,
            "search" => &mut self.tool_calls_search,
            "edit" => &mut self.tool_calls_edit,
            "execute" => &mut self.tool_calls_execute,
            _ => &mut self.tool_calls_other,
        };
        *bucket = bucket.saturating_add(d);
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
        let TimingPhase::Implement = phase;
        self.llm_wait = self.llm_wait.saturating_add(d);
        self.implement = self.implement.saturating_add(d);
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

pub use cost::record_completion_cost;
pub use lifecycle::{
    attach_kpop_engine_loop_run_timing, attach_kpop_engine_loop_run_timing_for_model,
    attach_new_run_timing, attach_new_run_timing_with_cost_policy, finalize_and_emit_run_timing,
    finalize_run_timing_json_only, persist_open_run_timing_json, record_backoff, record_llm,
};
pub use report::print_summary_from_run_dir;
pub use tokens::{
    note_acp_assistant_activity, note_acp_tool_call_completion, note_acp_tool_call_start,
    record_completion_step,
};

#[cfg(test)]
mod timing_tests;

#[cfg(test)]
mod timing_footnote_tests;

pub mod acp_post_run;
