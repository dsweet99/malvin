use std::collections::HashMap;
use std::fs;

use pi::auth::AuthStorage;
use pi::sdk::ModelRegistry;

use super::{
    cache_is_fresh, load_provider_cache, merge_registry_with_live, now_secs,
    refresh_pi_provider_caches_if_stale, save_provider_cache, PI_MODEL_CACHE_TTL,
};

#[test]
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

#[test]
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

#[test]
fn provider_cache_round_trip_and_freshness() {
    crate::test_utils::with_isolated_home(|_| {
        save_provider_cache("openrouter", &["a".into(), "b".into()]);
        let cache = load_provider_cache("openrouter").expect("cache");
        assert_eq!(cache.model_ids, vec!["a".to_string(), "b".to_string()]);
        assert!(cache_is_fresh(cache.fetched_at_secs));

        let stale_secs = now_secs().saturating_sub(PI_MODEL_CACHE_TTL.as_secs() + 60);
        assert!(!cache_is_fresh(stale_secs));
    });
}

#[test]
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

#[test]
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

#[test]
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

#[test]
fn load_provider_cache_rejects_invalid_json() {
    crate::test_utils::with_isolated_home(|_| {
        let dir = crate::workspace_paths::malvin_user_home_root().join("pi-model-cache");
        fs::create_dir_all(&dir).expect("cache dir");
        fs::write(dir.join("openrouter.json"), "not-json").expect("write bad cache");
        assert!(load_provider_cache("openrouter").is_none());
    });
}

#[test]
fn stale_cache_is_not_treated_as_fresh() {
    crate::test_utils::with_isolated_home(|_| {
        let dir = crate::workspace_paths::malvin_user_home_root().join("pi-model-cache");
        fs::create_dir_all(&dir).expect("cache dir");
        let stale_secs = now_secs().saturating_sub(PI_MODEL_CACHE_TTL.as_secs() + 60);
        fs::write(
            dir.join("openrouter.json"),
            format!(r#"{{"fetched_at_secs":{stale_secs},"model_ids":["stale"]}}"#),
        )
        .expect("write stale cache");
        let cache = load_provider_cache("openrouter").expect("cache");
        assert!(!cache_is_fresh(cache.fetched_at_secs));
    });
}
