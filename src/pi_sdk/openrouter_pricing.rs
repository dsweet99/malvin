use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use pi::provider::ModelCost;
use serde::Deserialize;

use super::cache_clock::{cache_fetched_at_is_fresh, unix_now_secs};
use super::models_refresh;

const OPENROUTER_MODELS_URL: &str = "https://openrouter.ai/api/v1/models";
const CACHE_TTL: Duration = Duration::from_hours(24);
const CACHE_FILE: &str = "openrouter-pricing.json";

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PricingCache {
    fetched_at_secs: u64,
    by_id: HashMap<String, ModelCost>,
}

fn cache_path() -> PathBuf {
    crate::workspace_paths::malvin_user_home_root().join(CACHE_FILE)
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

fn load_cache() -> Option<PricingCache> {
    let body = fs::read_to_string(cache_path()).ok()?;
    serde_json::from_str(&body).ok()
}

fn save_cache(by_id: HashMap<String, ModelCost>) {
    let path = cache_path();
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    let payload = PricingCache {
        fetched_at_secs: unix_now_secs(),
        by_id,
    };
    if let Ok(json) = serde_json::to_string(&payload) {
        if fs::write(&temp, json).is_ok() {
            let _ = fs::rename(temp, path);
        } else {
            let _ = fs::remove_file(temp);
        }
    }
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

async fn fetch_live_pricing_async(api_key: &str, url: &str) -> Option<HashMap<String, ModelCost>> {
    let client = pi::http::client::Client::new();
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Accept", "application/json")
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .ok()?;
    let status = response.status();
    if !(200..300).contains(&status) {
        return None;
    }
    let body = response.text().await.ok()?;
    pricing_from_models_body(&body)
}

fn fetch_live_pricing_sync(api_key: &str, url: &str) -> Option<HashMap<String, ModelCost>> {
    let Ok(runtime) = asupersync::runtime::RuntimeBuilder::current_thread().build() else {
        return None;
    };
    runtime.block_on(fetch_live_pricing_async(api_key, url))
}

/// Fetch `OpenRouter` `/models` pricing into `~/.malvin_home/openrouter-pricing.json`.
///
/// Safe to call before Pi starts; uses a dedicated current-thread `asupersync` runtime
/// and Pi's HTTP client (same sync-fetch pattern as `models_refresh`).
pub(crate) fn warm_openrouter_pricing_cache(force: bool) {
    if !force
        && let Some(cache) = load_cache()
        && cache_fetched_at_is_fresh(cache.fetched_at_secs, CACHE_TTL)
    {
        return;
    }
    let api_key = models_refresh::resolve_provider_api_key("openrouter");
    if api_key.trim().is_empty() {
        return;
    }
    let fetched = fetch_live_pricing_sync(&api_key, &models_url());
    if let Some(by_id) = fetched {
        save_cache(by_id);
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
    let cache = load_cache()?;
    for id in openrouter_lookup_ids(model_id) {
        if let Some(cost) = cache.by_id.get(&id) {
            return Some(cost.clone());
        }
    }
    None
}

#[cfg(test)]
#[path = "openrouter_pricing_tests.rs"]
mod openrouter_pricing_tests;
