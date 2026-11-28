use serde_json::{Map, Value};

use super::model_cost::ModelCost;

#[path = "portkey_body.rs"]
mod body;
#[path = "portkey_pricing_fetch.rs"]
mod fetch;

#[cfg(test)]
use body::{model_cost_from_body, pricing_model_path};
#[path = "portkey_route.rs"]
mod route;

use route::{CustomModelRoute, custom_model_route};

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

impl super::pricing_lookup::PricingSource for PortkeyLive {
    fn lookup(&self, provider: &str, model_id: &str) -> Option<ModelCost> {
        let upstream = portkey_upstream(provider, model_id)?;
        fetch::lookup_rates(&upstream, bare_model_id(model_id))
    }
}

pub(crate) fn uses_openrouter_catalog(provider: &str, model_id: &str) -> bool {
    provider.eq_ignore_ascii_case("openrouter")
        || openrouter_catalog_model(provider, model_id).is_some()
}

#[cfg(test)]
#[path = "portkey_pricing_tests.rs"]
mod portkey_pricing_tests;
