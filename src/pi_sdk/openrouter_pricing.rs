use std::collections::HashMap;
use std::time::Duration;

use serde::Deserialize;

use super::http_fetch::{HttpRequest, fetch_text};
use super::model_cost::ModelCost;
use super::pricing_cache_file::PricingCacheFile;

const OPENROUTER_MODELS_URL: &str = "https://openrouter.ai/api/v1/models";
const CACHE: PricingCacheFile =
    PricingCacheFile::new("openrouter-pricing.json", Duration::from_hours(24));

fn models_url() -> String {
    #[cfg(test)]
    if let Ok(url) = std::env::var("MALVIN_TEST_OPENROUTER_MODELS_URL") {
        return url;
    }
    OPENROUTER_MODELS_URL.to_string()
}

#[derive(Debug, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModelEntry>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModelEntry {
    id: String,
    pricing: OpenRouterPricing,
}

#[derive(Debug, Deserialize)]
struct OpenRouterPricing {
    prompt: String,
    completion: String,
    #[serde(default)]
    input_cache_read: Option<String>,
    #[serde(default)]
    input_cache_write: Option<String>,
}

fn parse_rate_per_million(value: &str) -> Option<f64> {
    let per_token = value.trim().parse::<f64>().ok()?;
    Some(per_token * 1_000_000.0)
}

fn model_cost_from_pricing(pricing: &OpenRouterPricing) -> Option<ModelCost> {
    Some(ModelCost {
        input: parse_rate_per_million(&pricing.prompt)?,
        output: parse_rate_per_million(&pricing.completion)?,
        cache_read: pricing
            .input_cache_read
            .as_deref()
            .and_then(parse_rate_per_million)
            .unwrap_or(0.0),
        cache_write: pricing
            .input_cache_write
            .as_deref()
            .and_then(parse_rate_per_million)
            .unwrap_or(0.0),
    })
}

fn pricing_from_models_body(body: &str) -> Option<HashMap<String, ModelCost>> {
    let parsed: OpenRouterModelsResponse = serde_json::from_str(body).ok()?;
    let mut by_id = HashMap::new();
    for entry in parsed.data {
        if let Some(cost) = model_cost_from_pricing(&entry.pricing) {
            by_id.insert(entry.id, cost);
        }
    }
    (!by_id.is_empty()).then_some(by_id)
}

fn fetch_live_pricing_sync(api_key: &str, url: &str) -> Option<HashMap<String, ModelCost>> {
    let headers = [("Authorization", format!("Bearer {api_key}"))];
    let body = fetch_text(&HttpRequest {
        url,
        headers: &headers,
        json_body: None,
        timeout: Duration::from_secs(15),
    })?;
    pricing_from_models_body(&body)
}

pub(crate) fn warm_openrouter_pricing_cache(force: bool) {
    if !force && CACHE.all_fresh() {
        return;
    }
    let api_key = super::auth::provider_api_key("openrouter").unwrap_or_default();
    if api_key.trim().is_empty() {
        return;
    }
    let fetched = fetch_live_pricing_sync(&api_key, &models_url());
    if let Some(by_id) = fetched {
        CACHE.replace_all(by_id);
    }
}

fn openrouter_lookup_ids(model_id: &str) -> Vec<String> {
    let trimmed = model_id.trim();
    let mut ids = vec![trimmed.to_string()];
    if trimmed == "auto" {
        ids.push("openrouter/auto".to_string());
    } else if !trimmed.contains('/') {
        ids.push(format!("openai/{trimmed}"));
    }
    if trimmed.starts_with('~') {
        ids.push(trimmed.trim_start_matches('~').to_string());
    }
    ids
}

pub(super) fn lookup_model_cost(model_id: &str) -> Option<ModelCost> {
    let by_id = CACHE.load();
    openrouter_lookup_ids(model_id)
        .iter()
        .find_map(|id| by_id.get(id).map(|entry| entry.cost.clone()))
}

#[cfg(test)]
pub(super) fn write_rate_cache_for_test(by_id: HashMap<String, ModelCost>) {
    CACHE.replace_all(by_id);
}

#[cfg(test)]
#[path = "openrouter_pricing_tests.rs"]
mod openrouter_pricing_tests;
