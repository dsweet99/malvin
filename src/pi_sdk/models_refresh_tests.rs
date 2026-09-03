use std::collections::HashMap;
use std::fs;

use pi::auth::AuthStorage;
use pi::sdk::ModelRegistry;

use super::super::cache_clock::{cache_fetched_at_is_fresh, unix_now_secs};
use super::{
    PI_MODEL_CACHE_TTL, load_provider_cache, merge_registry_with_live,
    refresh_pi_provider_caches_if_stale, save_provider_cache,
};

fn cursor_provider_is_excluded_from_live_fetch() {
    crate::acp::with_env("CURSOR_API_KEY", Some("test-key"), || {
        crate::test_utils::with_isolated_home(|_| {
            let live = refresh_pi_provider_caches_if_stale(true);
            assert!(
                !live.contains_key("cursor"),
                "cursor must not use Pi live fetch: {live:?}"
            );
        });
    });
}

fn live_fetch_skips_providers_without_api_key() {
    crate::test_utils::with_isolated_home(|_| {
        crate::acp::with_env("OPENAI_API_KEY", None, || {
            if !super::super::auth::is_provider_authenticated("openai") {
                return;
            }
            let live = refresh_pi_provider_caches_if_stale(true);
            assert!(
                !live.contains_key("openai"),
                "empty api key must skip live fetch: {live:?}"
            );
        });
    });
}

fn now_secs_returns_positive_epoch() {
    assert!(unix_now_secs() > 0);
}

fn resolve_provider_api_key_returns_empty_when_unconfigured() {
    crate::test_utils::with_isolated_home(|_| {
        crate::acp::with_env("OPENAI_API_KEY", None, || {
            if super::super::auth::is_provider_authenticated("openai") {
                return;
            }
            assert!(super::resolve_provider_api_key("openai").is_empty());
        });
    });
}
fn resolve_provider_api_key_reads_env_when_auth_missing() {
    crate::test_utils::with_isolated_home(|_| {
        crate::acp::with_env("OPENROUTER_API_KEY", Some("env-or-key"), || {
            assert_eq!(
                super::resolve_provider_api_key("openrouter").as_str(),
                "env-or-key"
            );
        });
    });
}

fn openai_compat_models_url_skips_non_openai_endpoints() {
    let google = pi::provider_metadata::provider_routing_defaults("google").expect("google");
    assert!(super::openai_compat_models_url(&google).is_none());
    let openrouter =
        pi::provider_metadata::provider_routing_defaults("openrouter").expect("openrouter");
    assert_eq!(
        super::openai_compat_models_url(&openrouter).as_deref(),
        Some("https://openrouter.ai/api/v1/models")
    );
}

fn provider_cache_round_trip_and_freshness() {
    crate::test_utils::with_isolated_home(|_| {
        save_provider_cache("openrouter", &["a".into(), "b".into()]);
        let cache = load_provider_cache("openrouter").expect("cache");
        assert_eq!(cache.model_ids, vec!["a".to_string(), "b".to_string()]);
        assert!(cache_fetched_at_is_fresh(
            cache.fetched_at_secs,
            PI_MODEL_CACHE_TTL
        ));

        let stale_secs = unix_now_secs().saturating_sub(PI_MODEL_CACHE_TTL.as_secs() + 60);
        assert!(!cache_fetched_at_is_fresh(stale_secs, PI_MODEL_CACHE_TTL));
    });
}

fn merge_registry_with_live_prefers_live_ids_and_enriches_metadata() {
    let auth = AuthStorage::load(pi::sdk::Config::auth_path()).expect("auth");
    let registry = ModelRegistry::load_for_listing(&auth, None);
    let mut live = HashMap::new();
    live.insert(
        "openrouter".to_string(),
        vec![
            "anthropic/claude-sonnet-4".into(),
            "vendor/only-live".into(),
        ],
    );
    let merged = merge_registry_with_live(&registry, &live);
    assert!(
        merged
            .iter()
            .any(|row| row.id == "openrouter/vendor/only-live" && row.name == "vendor/only-live"),
        "live-only ids must appear: {merged:?}"
    );
    assert!(
        merged
            .iter()
            .any(|row| row.id == "openrouter/anthropic/claude-sonnet-4"),
        "live ids must be kept: {merged:?}"
    );
}

fn refresh_skips_network_when_cache_is_fresh() {
    crate::test_utils::with_isolated_home(|_| {
        crate::acp::with_env("OPENROUTER_API_KEY", Some("test-key"), || {
            save_provider_cache("openrouter", &["cached-model".into()]);
            let live = refresh_pi_provider_caches_if_stale(false);
            assert_eq!(
                live.get("openrouter").map(std::vec::Vec::as_slice),
                Some(&["cached-model".to_string()][..])
            );
        });
    });
}

fn merge_registry_with_live_keeps_static_when_provider_not_refreshed() {
    let auth = AuthStorage::load(pi::sdk::Config::auth_path()).expect("auth");
    let registry = ModelRegistry::load_for_listing(&auth, None);
    let live = HashMap::new();
    let merged = merge_registry_with_live(&registry, &live);
    assert!(
        merged.iter().any(|row| row.id.contains('/')),
        "static registry rows must remain when no live provider cache: {merged:?}"
    );
}

fn load_provider_cache_rejects_invalid_json() {
    crate::test_utils::with_isolated_home(|_| {
        let dir = crate::workspace_paths::malvin_user_home_root().join("pi-model-cache");
        fs::create_dir_all(&dir).expect("cache dir");
        fs::write(dir.join("openrouter.json"), "not-json").expect("write bad cache");
        assert!(load_provider_cache("openrouter").is_none());
    });
}

fn stale_cache_is_not_treated_as_fresh() {
    crate::test_utils::with_isolated_home(|_| {
        let dir = crate::workspace_paths::malvin_user_home_root().join("pi-model-cache");
        fs::create_dir_all(&dir).expect("cache dir");
        let stale_secs = unix_now_secs().saturating_sub(PI_MODEL_CACHE_TTL.as_secs() + 60);
        fs::write(
            dir.join("openrouter.json"),
            format!(r#"{{"fetched_at_secs":{stale_secs},"model_ids":["stale"]}}"#),
        )
        .expect("write stale cache");
        let cache = load_provider_cache("openrouter").expect("cache");
        assert!(!cache_fetched_at_is_fresh(
            cache.fetched_at_secs,
            PI_MODEL_CACHE_TTL
        ));
    });
}

#[test]
fn kiss_bundled_pi_sdk_models_refresh_tests() {
    cursor_provider_is_excluded_from_live_fetch();
    live_fetch_skips_providers_without_api_key();
    now_secs_returns_positive_epoch();
    resolve_provider_api_key_returns_empty_when_unconfigured();
    resolve_provider_api_key_reads_env_when_auth_missing();
    openai_compat_models_url_skips_non_openai_endpoints();
    provider_cache_round_trip_and_freshness();
    merge_registry_with_live_prefers_live_ids_and_enriches_metadata();
    refresh_skips_network_when_cache_is_fresh();
    merge_registry_with_live_keeps_static_when_provider_not_refreshed();
    load_provider_cache_rejects_invalid_json();
    stale_cache_is_not_treated_as_fresh();
}
