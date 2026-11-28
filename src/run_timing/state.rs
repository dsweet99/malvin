use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::policy::{AcpStepProxy, CostPolicy, TimingPhase};

#[derive(Clone, Debug, Default)]
pub(crate) struct Ledger {
    pub(crate) llm_wait: Duration,
    pub(crate) tool_calls: Duration,
    pub(crate) read: Duration,
    pub(crate) search: Duration,
    pub(crate) edit: Duration,
    pub(crate) execute: Duration,
    pub(crate) other: Duration,
    pub(crate) steps: u64,
    pub(crate) tokens_in: Option<u64>,
    pub(crate) tokens_out: Option<u64>,
    pub(crate) cache_read: Option<u64>,
    pub(crate) cache_write: Option<u64>,
    pub(crate) reasoning: Option<u64>,
    pub(crate) cost_in: Option<f64>,
    pub(crate) cost_out: Option<f64>,
    pub(crate) cost_read: Option<f64>,
    pub(crate) cost_write: Option<f64>,
    pub(crate) cost_tot: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct RunTiming {
    pub(super) wall_start: Option<Instant>,
    pub(super) wall_end: Option<Instant>,
    pub(crate) llm_wait: Duration,
    pub(super) agent_retry_backoff: Duration,
    pub(super) implement: Duration,
    pub(super) implement_display_name: &'static str,
    pub(crate) tool_calls: Duration,
    pub(crate) tool_calls_read: Duration,
    pub(crate) tool_calls_search: Duration,
    pub(crate) tool_calls_edit: Duration,
    pub(crate) tool_calls_execute: Duration,
    pub(crate) tool_calls_other: Duration,
    pub(crate) ledger_cursor: Ledger,
    pub(crate) tx_costs: Vec<f64>,
    pub(crate) unknown_tx_count: u32,
    pub(crate) token_cost_rates: crate::config::malvin_config_file::TokenCostRates,
    pub(crate) cost_policy: CostPolicy,
    pub(crate) steps: u64,
    pub(crate) tokens_in: Option<u64>,
    pub(crate) tokens_out: Option<u64>,
    pub(crate) cache_read: Option<u64>,
    pub(crate) cache_write: Option<u64>,
    pub(crate) reasoning_tokens: Option<u64>,
    pub(crate) reported_cost_in: Option<f64>,
    pub(crate) reported_cost_out: Option<f64>,
    pub(crate) reported_cost_read: Option<f64>,
    pub(crate) reported_cost_write: Option<f64>,
    pub(crate) reported_cost_total: Option<f64>,
    pub(crate) estimated_cost_in: Option<f64>,
    pub(crate) estimated_cost_out: Option<f64>,
    pub(crate) estimated_cost_read: Option<f64>,
    pub(crate) estimated_cost_write: Option<f64>,
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
            ledger_cursor: Ledger::default(),
            tx_costs: Vec::new(),
            unknown_tx_count: 0,
            token_cost_rates: crate::config::malvin_config_file::TokenCostRates::default(),
            cost_policy: CostPolicy::EstimateFromRates,
            steps: 0,
            tokens_in: None,
            tokens_out: None,
            cache_read: None,
            cache_write: None,
            reasoning_tokens: None,
            reported_cost_in: None,
            reported_cost_out: None,
            reported_cost_read: None,
            reported_cost_write: None,
            reported_cost_total: None,
            estimated_cost_in: None,
            estimated_cost_out: None,
            estimated_cost_read: None,
            estimated_cost_write: None,
            tool_call_starts: 0,
            usage_tx_count: 0,
            unknown_usage_tx_count: 0,
            acp_step_proxy: AcpStepProxy::Idle,
        }
    }
}

impl RunTiming {
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
}
