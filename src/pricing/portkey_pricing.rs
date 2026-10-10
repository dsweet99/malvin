use serde::Deserialize;
use serde_json::{Map, Value};

use super::model_cost::{ModelCost, TokenUsage};

#[path = "portkey_pricing_fetch.rs"]
mod fetch;
#[path = "portkey_route.rs"]
mod route;

use route::{CustomModelRoute, custom_model_route};

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

fn header_provider(headers: &Map<String, Value>) -> Option<String> {
    let value = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("x-portkey-provider"))?
        .1
        .as_str()?
        .trim();
    if value.is_empty() || value.starts_with(['@', '$', '!']) {
        return None;
    }
    Some(value.to_string())
}

fn point_rate(point: Option<&PricePoint>) -> f64 {
    point.map_or(0.0, |p| p.price * 10_000.0)
}

pub(super) fn pricing_model_path(model: &str) -> String {
    model
        .split('/')
        .map(urlencoding)
        .collect::<Vec<_>>()
        .join("/")
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

fn route_is_portkey(route: &CustomModelRoute) -> bool {
    host_is_portkey(&route.base_url)
        || route
            .headers
            .keys()
            .any(|name| name_is_portkey_header(name))
}

fn portkey_upstream(provider: &str, model_id: &str) -> Option<String> {
    let route = custom_model_route(provider, model_id)?;
    if !route_is_portkey(&route) {
        return None;
    }
    upstream_name(provider, header_provider(&route.headers))
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
    let upstream = portkey_upstream(provider, model_id)?;
    upstream
        .eq_ignore_ascii_case("openrouter")
        .then(|| bare_model_id(model_id).to_string())
}

pub(super) struct PortkeyLive;

impl super::pricing_source::PricingSource for PortkeyLive {
    fn lookup(&self, provider: &str, model_id: &str) -> Option<ModelCost> {
        let upstream = portkey_upstream(provider, model_id)?;
        fetch::lookup_rates(&upstream, bare_model_id(model_id))
    }
}

pub(crate) fn uses_openrouter_catalog(provider: &str, model_id: &str) -> bool {
    provider.eq_ignore_ascii_case("openrouter")
        || openrouter_catalog_model(provider, model_id).is_some()
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
    let Some(rates) = super::pricing_source::lookup_rates(provider, model) else {
        return;
    };
    let Some(obj) = usage.as_object() else {
        return;
    };
    let cost = rates.cost_for(TokenUsage {
        input: token_count(obj, &["inputTokens", "input"]),
        output: token_count(obj, &["outputTokens", "output"]),
        cache_read: token_count(obj, &["cacheReadTokens", "cacheRead", "cache_read"]),
        cache_write: token_count(obj, &["cacheWriteTokens", "cacheWrite", "cache_write"]),
    });
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
