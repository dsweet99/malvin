use std::collections::HashMap;

use pi::provider::ModelCost;
use pi::sdk::{Config, ModelRegistry};
use serde::Deserialize;
use serde_json::{Map, Value};

#[path = "portkey_pricing_fetch.rs"]
mod fetch;

#[derive(Debug, Deserialize)]
struct PricingBody {
    pay_as_you_go: Option<PayGo>,
}

#[derive(Debug, Deserialize)]
struct PayGo {
    #[serde(rename = "request_token")]
    request: Option<PricePoint>,
    #[serde(rename = "response_token")]
    response: Option<PricePoint>,
    #[serde(rename = "cache_write_input_token")]
    cache_write: Option<PricePoint>,
    #[serde(rename = "cache_read_input_token")]
    cache_read: Option<PricePoint>,
}

#[derive(Debug, Deserialize)]
struct PricePoint {
    price: f64,
}

pub(super) fn host_is_portkey(base_url: &str) -> bool {
    let rest = base_url
        .trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://");
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let host = host.split('@').next_back().unwrap_or(host);
    let host = host.split(':').next().unwrap_or(host);
    host.eq_ignore_ascii_case("api.portkey.ai")
}

pub(super) fn name_is_portkey_header(name: &str) -> bool {
    name.to_ascii_lowercase().starts_with("x-portkey-")
}

fn bare_model_id(model_id: &str) -> &str {
    let id = model_id.trim();
    let Some(rest) = id.strip_prefix('@') else {
        return id;
    };
    rest.split_once('/').map_or(id, |(_, model)| model)
}

fn header_provider(headers: &HashMap<String, String>) -> Option<String> {
    let value = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("x-portkey-provider"))?
        .1
        .trim();
    if value.is_empty() || value.starts_with(['@', '$', '!']) {
        return None;
    }
    Some(value.to_string())
}

fn point_rate(point: Option<&PricePoint>) -> f64 {
    point.map_or(0.0, |p| p.price * 10_000.0)
}

pub(super) fn model_cost_from_body(body: &str) -> Option<ModelCost> {
    let parsed: PricingBody = serde_json::from_str(body).ok()?;
    let pay = parsed.pay_as_you_go?;
    let cost = ModelCost {
        input: point_rate(pay.request.as_ref()),
        output: point_rate(pay.response.as_ref()),
        cache_read: point_rate(pay.cache_read.as_ref()),
        cache_write: point_rate(pay.cache_write.as_ref()),
    };
    cost.is_priced().then_some(cost)
}

fn load_registry() -> Option<ModelRegistry> {
    let auth = pi::auth::AuthStorage::load(Config::auth_path()).ok()?;
    let path = pi::models::default_models_path(&Config::global_dir());
    Some(ModelRegistry::load(&auth, Some(path)))
}

fn entry_header_provider(entry: &pi::sdk::ModelEntry) -> Option<String> {
    header_provider(&entry.headers).or_else(|| header_provider(&entry.model.headers))
}

fn entry_is_portkey(entry: &pi::sdk::ModelEntry) -> bool {
    if host_is_portkey(&entry.model.base_url) {
        return true;
    }
    entry
        .headers
        .keys()
        .any(|name| name_is_portkey_header(name))
        || entry
            .model
            .headers
            .keys()
            .any(|name| name_is_portkey_header(name))
}

fn upstream_name(pi_provider: &str, from_header: Option<String>) -> Option<String> {
    if let Some(name) = from_header {
        return Some(name);
    }
    if pi_provider.eq_ignore_ascii_case("portkey") {
        None
    } else {
        Some(pi_provider.to_string())
    }
}

pub(super) fn openrouter_catalog_model(provider: &str, model_id: &str) -> Option<String> {
    let registry = load_registry()?;
    let entry = registry.find(provider, model_id)?;
    if !entry_is_portkey(&entry) {
        return None;
    }
    let upstream = upstream_name(provider, entry_header_provider(&entry))?;
    if !upstream.eq_ignore_ascii_case("openrouter") {
        return None;
    }
    Some(bare_model_id(model_id).to_string())
}

pub(super) fn rates_for_pi_model(provider: &str, model_id: &str) -> Option<ModelCost> {
    let registry = load_registry()?;
    let entry = registry.find(provider, model_id)?;
    if !entry_is_portkey(&entry) {
        return None;
    }
    let upstream = upstream_name(provider, entry_header_provider(&entry))?;
    fetch::lookup_rates(&upstream, bare_model_id(model_id))
}

fn token_count(obj: &Map<String, Value>, keys: &[&str]) -> u64 {
    keys.iter()
        .find_map(|key| obj.get(*key).and_then(Value::as_u64))
        .unwrap_or(0)
}

pub(crate) fn apply_portkey_cost_usd(provider: &str, model: &str, usage: &mut Value) {
    super::sdk_usage_fields::normalize_pi_usage(usage);
    if super::sdk_usage_fields::cost_usd_is_positive(usage) {
        return;
    }
    let Some(rates) = rates_for_pi_model(provider, model)
        .or_else(|| super::usage_cost::rates_for_provider_model(provider, model))
    else {
        return;
    };
    let Some(obj) = usage.as_object() else {
        return;
    };
    let usage_tokens = pi::model::Usage {
        input: token_count(obj, &["inputTokens", "input"]),
        output: token_count(obj, &["outputTokens", "output"]),
        cache_read: token_count(obj, &["cacheReadTokens", "cacheRead", "cache_read"]),
        cache_write: token_count(obj, &["cacheWriteTokens", "cacheWrite", "cache_write"]),
        ..pi::model::Usage::default()
    };
    let cost = super::usage_cost::cost_from_model_rates(&rates, &usage_tokens);
    if let Some(map) = usage.as_object_mut() {
        map.insert(
            "costUsd".into(),
            serde_json::json!({
                "input": cost.input,
                "output": cost.output,
                "cacheRead": cost.cache_read,
                "cacheWrite": cost.cache_write,
                "total": cost.total,
            }),
        );
    }
}

#[cfg(test)]
#[path = "portkey_pricing_tests.rs"]
mod portkey_pricing_tests;
