use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use pi::provider::ModelCost;
use serde::Deserialize;

use super::super::cache_clock::{cache_fetched_at_is_fresh, unix_now_secs};
use super::super::openrouter_billed_cost::urlencoding;
use super::model_cost_from_body;

const PRICING_URL: &str = "https://api.portkey.ai/model-configs/pricing";
const CACHE_TTL: Duration = Duration::from_hours(24);
const CACHE_FILE: &str = "portkey-pricing.json";

#[derive(Debug, Clone, serde::Serialize, Deserialize)]
struct PricingCache {
    fetched_at_secs: u64,
    by_id: HashMap<String, ModelCost>,
}

fn cache_path() -> PathBuf {
    crate::workspace_paths::malvin_user_home_root().join(CACHE_FILE)
}

fn load_cache() -> Option<PricingCache> {
    let body = fs::read_to_string(cache_path()).ok()?;
    serde_json::from_str(&body).ok()
}

fn save_cache(cache: &PricingCache) {
    let path = cache_path();
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    if let Ok(json) = serde_json::to_string(cache) {
        if fs::write(&temp, json).is_ok() {
            let _ = fs::rename(temp, path);
        } else {
            let _ = fs::remove_file(temp);
        }
    }
}

fn pricing_url(provider: &str, model: &str) -> String {
    #[cfg(test)]
    if let Ok(url) = std::env::var("MALVIN_TEST_PORTKEY_PRICING_URL") {
        return url;
    }
    format!(
        "{PRICING_URL}/{}/{}",
        urlencoding(provider),
        urlencoding(model)
    )
}

#[allow(clippy::missing_const_for_fn)]
fn live_fetch_allowed() -> bool {
    #[cfg(test)]
    {
        std::env::var_os("MALVIN_TEST_PORTKEY_PRICING_URL").is_some()
    }
    #[cfg(not(test))]
    {
        true
    }
}

pub(super) fn lookup_rates(provider: &str, model: &str) -> Option<ModelCost> {
    let key = format!("{provider}/{model}");
    if let Some(cached) = fresh_cached_rate(&key) {
        return Some(cached);
    }
    if !live_fetch_allowed() {
        return None;
    }
    let body = fetch_pricing_body(&pricing_url(provider, model))?;
    let cost = model_cost_from_body(&body)?;
    store_rate(&key, cost.clone());
    Some(cost)
}

fn fresh_cached_rate(key: &str) -> Option<ModelCost> {
    let cache = load_cache()?;
    if !cache_fetched_at_is_fresh(cache.fetched_at_secs, CACHE_TTL) {
        return None;
    }
    cache.by_id.get(key).cloned()
}

fn store_rate(key: &str, cost: ModelCost) {
    let mut cache = load_cache().unwrap_or_else(|| PricingCache {
        fetched_at_secs: unix_now_secs(),
        by_id: HashMap::new(),
    });
    cache.fetched_at_secs = unix_now_secs();
    cache.by_id.insert(key.to_string(), cost);
    save_cache(&cache);
}

fn fetch_pricing_body(url: &str) -> Option<String> {
    let Ok(runtime) = asupersync::runtime::RuntimeBuilder::current_thread().build() else {
        return None;
    };
    runtime.block_on(fetch_pricing_body_async(url))
}

async fn fetch_pricing_body_async(url: &str) -> Option<String> {
    let client = pi::http::client::Client::new();
    let response = client
        .get(url)
        .header("Accept", "application/json")
        .timeout(Duration::from_secs(15))
        .send()
        .await
        .ok()?;
    if !(200..300).contains(&response.status()) {
        return None;
    }
    response.text().await.ok()
}
