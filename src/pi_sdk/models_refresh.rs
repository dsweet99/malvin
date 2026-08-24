use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use pi::auth::AuthStorage;
use pi::sdk::{Config, ModelRegistry};
use serde::{Deserialize, Serialize};

use super::auth::is_provider_authenticated;
use super::models_list::PiModelListing;

pub const PI_MODEL_CACHE_TTL: Duration = Duration::from_hours(24);

const PI_MODEL_CACHE_DIR: &str = "pi-model-cache";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProviderModelCache {
    fetched_at_secs: u64,
    model_ids: Vec<String>,
}

fn pi_model_cache_dir() -> PathBuf {
    crate::workspace_paths::malvin_user_home_root().join(PI_MODEL_CACHE_DIR)
}

fn pi_model_cache_path(provider: &str) -> PathBuf {
    pi_model_cache_dir().join(format!("{provider}.json"))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn cache_is_fresh(fetched_at_secs: u64) -> bool {
    now_secs().saturating_sub(fetched_at_secs) < PI_MODEL_CACHE_TTL.as_secs()
}

pub(crate) fn load_provider_cache(provider: &str) -> Option<ProviderModelCache> {
    let body = fs::read_to_string(pi_model_cache_path(provider)).ok()?;
    serde_json::from_str(&body).ok()
}

pub(crate) fn save_provider_cache(provider: &str, model_ids: &[String]) {
    let dir = pi_model_cache_dir();
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let payload = ProviderModelCache {
        fetched_at_secs: now_secs(),
        model_ids: model_ids.to_vec(),
    };
    let path = pi_model_cache_path(provider);
    let temp = path.with_extension(format!("tmp-{}", std::process::id()));
    if let Ok(json) = serde_json::to_string(&payload) {
        if fs::write(&temp, json).is_ok() {
            let _ = fs::rename(temp, path);
        } else {
            let _ = fs::remove_file(temp);
        }
    }
}

fn resolve_provider_api_key(provider: &str) -> String {
    if let Ok(auth) = AuthStorage::load(Config::auth_path())
        && let Some(key) = auth.api_key(provider)
        && !key.trim().is_empty()
    {
        return key;
    }
    for env_key in pi::provider_metadata::provider_auth_env_keys(provider) {
        if let Ok(value) = std::env::var(env_key)
            && !value.trim().is_empty()
        {
            return value;
        }
    }
    String::new()
}

fn authenticated_providers() -> Vec<&'static str> {
    pi::provider_metadata::PROVIDER_METADATA
        .iter()
        .map(|meta| meta.canonical_id)
        .filter(|id| is_provider_authenticated(id))
        .collect()
}

/// Whether Pi's OpenAI-compatible `/v1/models` probe can succeed for this provider.
///
/// Cursor is excluded: malvin lists Cursor models via `cursor-sdk-bridge`, and Pi's
/// generic `{base_url}/models` probe against Cursor's `AgentService` endpoint returns
/// HTTP 464 instead of a model catalog.
fn provider_supports_pi_live_model_fetch(provider: &str) -> bool {
    if provider.eq_ignore_ascii_case("cursor") {
        return false;
    }
    let Some(defaults) = pi::provider_metadata::provider_routing_defaults(provider) else {
        return false;
    };
    openai_compat_models_url(&defaults).is_some()
}

fn openai_compat_models_url(defaults: &pi::provider_metadata::ProviderRoutingDefaults) -> Option<String> {
    let base = defaults.base_url.trim_end_matches('/');
    if base.is_empty() {
        return None;
    }
    if base.ends_with("/messages") || base.contains("/v1beta") || base.contains("googleapis.com") {
        return None;
    }
    Some(format!("{base}/models"))
}

fn provider_needs_refresh(provider: &str, force: bool) -> bool {
    if force {
        return true;
    }
    !matches!(
        load_provider_cache(provider),
        Some(cache) if cache_is_fresh(cache.fetched_at_secs)
    )
}

fn fetch_provider_models_sync(provider: &str, force: bool) -> Vec<String> {
    let api_key = resolve_provider_api_key(provider);
    let Ok(runtime) = asupersync::runtime::RuntimeBuilder::current_thread().build() else {
        return Vec::new();
    };
    let result = runtime.block_on(async {
        if force {
            pi::providers::refresh_provider_models(provider, &api_key).await
        } else {
            pi::providers::fetch_provider_models(provider, &api_key).await
        }
    });
    result.unwrap_or_default()
}

pub(crate) fn refresh_pi_provider_caches_if_stale(force: bool) -> HashMap<String, Vec<String>> {
    let mut live = HashMap::new();
    for provider in authenticated_providers() {
        if !provider_supports_pi_live_model_fetch(provider)
            || resolve_provider_api_key(provider).trim().is_empty()
        {
            continue;
        }
        if provider_needs_refresh(provider, force) {
            let ids = fetch_provider_models_sync(provider, true);
            if !ids.is_empty() {
                save_provider_cache(provider, &ids);
                live.insert(provider.to_string(), ids);
            }
            continue;
        }
        if let Some(cache) = load_provider_cache(provider)
            && !cache.model_ids.is_empty()
        {
            live.insert(provider.to_string(), cache.model_ids);
        }
    }
    live
}

pub(crate) fn merge_registry_with_live(
    registry: &ModelRegistry,
    live_by_provider: &HashMap<String, Vec<String>>,
) -> Vec<PiModelListing> {
    let static_by_key = static_registry_lookup(registry);
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    append_live_models(&mut out, &mut seen, &static_by_key, live_by_provider);
    append_static_models_without_live(&mut out, &mut seen, registry, live_by_provider);
    out
}

fn static_registry_lookup(
    registry: &ModelRegistry,
) -> HashMap<(String, String), &pi::models::ModelEntry> {
    registry
        .models()
        .iter()
        .map(|entry| {
            (
                (entry.model.provider.clone(), entry.model.id.clone()),
                entry,
            )
        })
        .collect()
}

fn append_live_models(
    out: &mut Vec<PiModelListing>,
    seen: &mut HashSet<String>,
    static_by_key: &HashMap<(String, String), &pi::models::ModelEntry>,
    live_by_provider: &HashMap<String, Vec<String>>,
) {
    for (provider, ids) in live_by_provider {
        for id in ids {
            let full_id = format!("{provider}/{id}");
            if !seen.insert(full_id.clone()) {
                continue;
            }
            let (name, thinking) = static_by_key
                .get(&(provider.clone(), id.clone()))
                .map_or_else(
                    || (id.clone(), None),
                    |entry| (entry.model.name.clone(), Some(entry.model.reasoning)),
                );
            out.push(PiModelListing {
                id: full_id,
                name,
                thinking,
            });
        }
    }
}

fn append_static_models_without_live(
    out: &mut Vec<PiModelListing>,
    seen: &mut HashSet<String>,
    registry: &ModelRegistry,
    live_by_provider: &HashMap<String, Vec<String>>,
) {
    for entry in registry.models() {
        let provider = entry.model.provider.as_str();
        if live_by_provider.contains_key(provider) {
            continue;
        }
        let full_id = format!("{provider}/{}", entry.model.id);
        if seen.insert(full_id.clone()) {
            out.push(PiModelListing {
                id: full_id,
                name: entry.model.name.clone(),
                thinking: Some(entry.model.reasoning),
            });
        }
    }
}

#[cfg(test)]
#[path = "models_refresh_tests.rs"]
mod models_refresh_tests;
