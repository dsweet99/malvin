use crate::llm_transport::ResponseUsage;

use super::acp_usage::{
    AcpUsageFields, add_optional_sum, reported_cost_usd, u64_field, usage_payload_is_observable,
};
use super::{AcpStepProxy, RunTiming};

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
        let fields = AcpUsageFields {
            input: u64_field(obj, "inputTokens"),
            output: u64_field(obj, "outputTokens"),
            cache_read: u64_field(obj, "cacheReadTokens"),
            cache_write: u64_field(obj, "cacheWriteTokens"),
            reasoning: u64_field(obj, "reasoningTokens"),
            total: u64_field(obj, "totalTokens"),
            skip_rate_estimate: false,
        };
        let reported = reported_cost_usd(obj);
        if !usage_payload_is_observable(obj) {
            return;
        }
        let fields = AcpUsageFields {
            skip_rate_estimate: reported.is_some(),
            ..fields
        };
        self.apply_acp_usage_fields(fields);
        if let Some(cost) = reported {
            self.record_reported_cost_usd(&cost);
        }
    }
}

pub(crate) fn acp_usage_payload_is_observable(usage: &serde_json::Value) -> bool {
    usage.as_object().is_some_and(usage_payload_is_observable)
}

fn optional_u64_json(v: Option<u64>) -> serde_json::Value {
    v.map_or(serde_json::Value::Null, |n| serde_json::json!(n))
}

fn insert_optional_token_count(
    obj: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<u64>,
) {
    if let Some(n) = value {
        obj.insert(key.into(), serde_json::json!(n));
    }
}

#[must_use]
pub fn tokens_stats(r: &RunTiming) -> serde_json::Value {
    let mut obj = serde_json::Map::new();
    obj.insert("steps".into(), serde_json::json!(r.steps));
    obj.insert("tokens_in".into(), optional_u64_json(r.tokens_in));
    obj.insert("tokens_out".into(), optional_u64_json(r.tokens_out));
    insert_optional_token_count(&mut obj, "cache_read", r.cache_read);
    insert_optional_token_count(&mut obj, "cache_write", r.cache_write);
    insert_optional_token_count(&mut obj, "reasoning", r.reasoning_tokens);
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
