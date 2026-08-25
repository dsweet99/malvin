use std::time::Duration;

use pi::model::Cost;
use serde::Deserialize;

use super::models_refresh;
use super::usage_cost::AggregatedCostUsd;

const OPENROUTER_GENERATION_URL: &str = "https://openrouter.ai/api/v1/generation";

#[derive(Debug, Deserialize)]
pub(super) struct GenerationResponse {
    data: GenerationData,
}

#[derive(Debug, Deserialize)]
pub(super) struct GenerationData {
    #[serde(default)]
    usage: Option<f64>,
    #[serde(default)]
    total_cost: Option<f64>,
    #[serde(default)]
    upstream_inference_cost: Option<f64>,
}

pub(super) fn cost_from_generation_data(data: &GenerationData) -> Option<Cost> {
    let total = data
        .usage
        .filter(|v| *v > 0.0)
        .or_else(|| data.total_cost.filter(|v| *v > 0.0))
        .or_else(|| data.upstream_inference_cost.filter(|v| *v > 0.0))?;
    Some(Cost {
        total,
        ..Cost::default()
    })
}

async fn fetch_generation_cost_async(api_key: &str, generation_id: &str) -> Option<Cost> {
    let client = pi::http::client::Client::new();
    let url = format!("{OPENROUTER_GENERATION_URL}?id={}", urlencoding(generation_id));
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Accept", "application/json")
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .ok()?;
    if !(200..300).contains(&response.status()) {
        return None;
    }
    let body = response.text().await.ok()?;
    let parsed: GenerationResponse = serde_json::from_str(&body).ok()?;
    cost_from_generation_data(&parsed.data)
}

pub(super) fn urlencoding(value: &str) -> String {
    value
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect()
}

fn fetch_generation_cost_sync(api_key: &str, generation_id: &str) -> Option<Cost> {
    let Ok(runtime) = asupersync::runtime::RuntimeBuilder::current_thread().build() else {
        return None;
    };
    runtime.block_on(fetch_generation_cost_async(api_key, generation_id))
}

/// Sum billed `OpenRouter` costs for generation ids captured during the `Pi` stream.
pub(super) fn fetch_billed_cost_from_generation_ids() -> Option<AggregatedCostUsd> {
    let ids = take_openrouter_generation_ids();
    if ids.is_empty() {
        return None;
    }
    let api_key = models_refresh::resolve_provider_api_key("openrouter");
    if api_key.trim().is_empty() {
        return None;
    }
    let mut totals = AggregatedCostUsd::default();
    for id in ids {
        if let Some(cost) = fetch_generation_cost_sync(&api_key, &id) {
            totals.absorb(&cost);
        }
    }
    totals.is_present().then_some(totals)
}

#[cfg(malvin_pi_openrouter_patch)]
fn take_openrouter_generation_ids() -> Vec<String> {
    pi::providers::openai::take_openrouter_generation_ids()
}

#[cfg(not(malvin_pi_openrouter_patch))]
fn take_openrouter_generation_ids() -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
#[path = "openrouter_billed_cost_tests.rs"]
mod openrouter_billed_cost_tests;
