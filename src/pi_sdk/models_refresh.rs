use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use super::cache_clock::{cache_fetched_at_is_fresh, unix_now_secs};
use pi::auth::AuthStorage;
use pi::sdk::Config;
use serde::{Deserialize, Serialize};

use super::auth::is_provider_authenticated;
use super::local_endpoint::keyless_local_provider_is_listening;

pub(crate) use super::models_refresh_merge::merge_registry_with_live;

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
        fetched_at_secs: unix_now_secs(),
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

pub(crate) fn resolve_provider_api_key(provider: &str) -> String {
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
    if pi::provider_metadata::provider_is_keyless_local(provider) {
        return super::local_context::KEYLESS_LOCAL_API_KEY.to_string();
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

fn provider_supports_pi_live_model_fetch(provider: &str) -> bool {
    if provider.eq_ignore_ascii_case("cursor") {
        return false;
    }
    let Some(defaults) = pi::provider_metadata::provider_routing_defaults(provider) else {
        return false;
    };
    openai_compat_models_url(&defaults).is_some()
}

fn openai_compat_models_url(
    defaults: &pi::provider_metadata::ProviderRoutingDefaults,
) -> Option<String> {
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
    if pi::provider_metadata::provider_is_keyless_local(provider) {
        return true;
    }
    !matches!(
        load_provider_cache(provider),
        Some(cache) if cache_fetched_at_is_fresh(cache.fetched_at_secs, PI_MODEL_CACHE_TTL)
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

fn insert_live_ids(live: &mut HashMap<String, Vec<String>>, provider: &str, ids: Vec<String>) {
    live.insert(provider.to_string(), ids);
}

fn record_fetched_provider(
    live: &mut HashMap<String, Vec<String>>,
    provider: &str,
    keyless: bool,
    ids: Vec<String>,
) {
    if keyless || !ids.is_empty() {
        save_provider_cache(provider, &ids);
        insert_live_ids(live, provider, ids);
    }
}

fn record_cached_provider(live: &mut HashMap<String, Vec<String>>, provider: &str, keyless: bool) {
    if let Some(cache) = load_provider_cache(provider)
        && !cache.model_ids.is_empty()
    {
        insert_live_ids(live, provider, cache.model_ids);
    } else if keyless {
        insert_live_ids(live, provider, Vec::new());
    }
}

fn refresh_one_provider(live: &mut HashMap<String, Vec<String>>, provider: &str, force: bool) {
    if !provider_supports_pi_live_model_fetch(provider) {
        return;
    }
    let api_key = resolve_provider_api_key(provider);
    let keyless = pi::provider_metadata::provider_is_keyless_local(provider);
    if api_key.trim().is_empty() && !keyless {
        return;
    }
    if keyless && !keyless_local_provider_is_listening(provider) {
        insert_live_ids(live, provider, Vec::new());
        return;
    }
    if provider_needs_refresh(provider, force) {
        record_fetched_provider(live, provider, keyless, fetch_provider_models_sync(provider, true));
        return;
    }
    record_cached_provider(live, provider, keyless);
}

pub fn refresh_pi_provider_caches_if_stale(force: bool) -> HashMap<String, Vec<String>> {
    let mut live = HashMap::new();
    for provider in authenticated_providers() {
        refresh_one_provider(&mut live, provider, force);
    }
    live
}

#[cfg(test)]
#[path = "models_refresh_tests.rs"]
mod models_refresh_tests;

#[cfg(test)]
#[path = "models_refresh_cache_tests.rs"]
mod models_refresh_cache_tests;
