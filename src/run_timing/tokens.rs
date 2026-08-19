use crate::llm_transport::ResponseUsage;

use super::{AcpStepProxy, RunTiming};

fn u64_field(obj: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<u64> {
    obj.get(key).and_then(serde_json::Value::as_u64)
}

fn add_optional_sum(slot: &mut Option<u64>, n: u64) {
    *slot = Some(slot.unwrap_or(0).saturating_add(n));
}

impl RunTiming {
    pub fn record_completion_step(&mut self, usage: Option<&ResponseUsage>) {
        self.steps = self.steps.saturating_add(1);
        match usage {
            Some(u) => {
                self.usage_tx_count = self.usage_tx_count.saturating_add(1);
                self.record_token_fields(u);
            }
            None => {
                self.unknown_usage_tx_count = self.unknown_usage_tx_count.saturating_add(1);
            }
        }
        if matches!(self.cost_policy, super::CostPolicy::Zero) {
            self.tx_costs.push(0.0);
        }
    }

    pub(crate) fn record_token_fields(&mut self, usage: &ResponseUsage) {
        if let Some(n) = usage.prompt_tokens {
            add_optional_sum(&mut self.tokens_in, n);
        }
        if let Some(n) = usage.completion_tokens {
            add_optional_sum(&mut self.tokens_out, n);
        }
    }

    pub const fn note_acp_tool_call_start(&mut self) {
        self.tool_call_starts = self.tool_call_starts.saturating_add(1);
        self.acp_step_proxy = AcpStepProxy::OpenBatch;
    }

    pub const fn note_acp_tool_call_completion(&mut self) {
        if matches!(self.acp_step_proxy, AcpStepProxy::OpenBatch) {
            self.steps = self.steps.saturating_add(1);
            self.acp_step_proxy = AcpStepProxy::Idle;
        }
    }

    pub const fn note_acp_assistant_activity(&mut self) {
        if !matches!(self.acp_step_proxy, AcpStepProxy::OpenBatch) {
            self.acp_step_proxy = AcpStepProxy::TrailingAssistant;
        }
    }

    pub const fn finalize_acp_trailing_assistant_step(&mut self) {
        if matches!(self.acp_step_proxy, AcpStepProxy::TrailingAssistant) {
            self.steps = self.steps.saturating_add(1);
            self.acp_step_proxy = AcpStepProxy::Idle;
        }
    }

    pub fn record_acp_usage_if_present(&mut self, usage: &serde_json::Value) {
        let Some(obj) = usage.as_object() else {
            return;
        };
        let input = u64_field(obj, "inputTokens");
        let output = u64_field(obj, "outputTokens");
        let cache_read = u64_field(obj, "cacheReadTokens");
        let cache_write = u64_field(obj, "cacheWriteTokens");
        if input.is_none() && output.is_none() && cache_read.is_none() && cache_write.is_none() {
            return;
        }
        self.apply_acp_usage_fields(input, output, cache_read, cache_write);
    }

    fn apply_acp_usage_fields(
        &mut self,
        input: Option<u64>,
        output: Option<u64>,
        cache_read: Option<u64>,
        cache_write: Option<u64>,
    ) {
        self.usage_tx_count = self.usage_tx_count.saturating_add(1);
        let input_n = input.unwrap_or(0);
        let output_n = output.unwrap_or(0);
        let cache_read_n = cache_read.unwrap_or(0);
        let cache_write_n = cache_write.unwrap_or(0);
        let tokens_in = input_n + cache_read_n + cache_write_n;
        if input.is_some() || cache_read.is_some() || cache_write.is_some() {
            add_optional_sum(&mut self.tokens_in, tokens_in);
        }
        if let Some(n) = cache_read {
            add_optional_sum(&mut self.cache_read, n);
        }
        if let Some(n) = cache_write {
            add_optional_sum(&mut self.cache_write, n);
        }
        if let Some(n) = output {
            add_optional_sum(&mut self.tokens_out, n);
        }
        if matches!(self.cost_policy, super::CostPolicy::EstimateFromRates) {
            let estimated =
                self.token_cost_rates
                    .estimate_usd(input_n, output_n, cache_read_n, cache_write_n);
            self.tx_costs.push(estimated);
        } else if matches!(self.cost_policy, super::CostPolicy::Zero) {
            self.tx_costs.push(0.0);
        }
    }
}

fn optional_u64_json(v: Option<u64>) -> serde_json::Value {
    v.map_or(serde_json::Value::Null, |n| serde_json::json!(n))
}

#[must_use]
pub fn tokens_stats(r: &RunTiming) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert("steps".into(), serde_json::json!(r.steps));
    obj.insert("tokens_in".into(), optional_u64_json(r.tokens_in));
    obj.insert("tokens_out".into(), optional_u64_json(r.tokens_out));
    if let Some(n) = r.cache_read {
        obj.insert("cache_read".into(), serde_json::json!(n));
    }
    if let Some(n) = r.cache_write {
        obj.insert("cache_write".into(), serde_json::json!(n));
    }
    obj.insert("usage_tx_count".into(), serde_json::json!(r.usage_tx_count));
    obj.insert(
        "unknown_usage_tx_count".into(),
        serde_json::json!(r.unknown_usage_tx_count),
    );
    if r.tool_call_starts > 0 {
        obj.insert(
            "tool_call_starts".into(),
            serde_json::json!(r.tool_call_starts),
        );
    }
    serde_json::Value::Object(obj)
}

pub fn record_completion_step(
    timing: Option<&std::sync::Arc<std::sync::Mutex<RunTiming>>>,
    usage: Option<&ResponseUsage>,
) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.record_completion_step(usage);
}

pub fn note_acp_tool_call_start(timing: Option<&std::sync::Arc<std::sync::Mutex<RunTiming>>>) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.note_acp_tool_call_start();
}

pub fn note_acp_tool_call_completion(timing: Option<&std::sync::Arc<std::sync::Mutex<RunTiming>>>) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.note_acp_tool_call_completion();
}

pub fn note_acp_assistant_activity(timing: Option<&std::sync::Arc<std::sync::Mutex<RunTiming>>>) {
    let Some(t) = timing else {
        return;
    };
    let mut g = t.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    g.note_acp_assistant_activity();
}

#[cfg(test)]
#[path = "tokens_tests.rs"]
mod tokens_tests;
