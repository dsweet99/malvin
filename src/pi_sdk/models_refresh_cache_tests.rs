use std::collections::HashMap;
use std::fs;

use pi::auth::AuthStorage;
use pi::sdk::ModelRegistry;

use super::super::cache_clock::{cache_fetched_at_is_fresh, unix_now_secs};
use super::{
    PI_MODEL_CACHE_TTL, load_provider_cache, merge_registry_with_live,
    refresh_pi_provider_caches_if_stale, save_provider_cache,
};

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
            save_provider_cache("ollama", &["stale-tag".into()]);
            assert!(super::provider_needs_refresh("ollama", false));
            assert!(!super::provider_needs_refresh("openrouter", false));
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

fn empty_live_keyless_suppresses_static_ghost_rows() {
    crate::test_utils::with_isolated_home(|_| {
        let agent = pi::sdk::Config::global_dir();
        fs::create_dir_all(&agent).expect("agent");
        let path = pi::models::default_models_path(&agent);
        fs::write(
            &path,
            r#"{"providers":{"ollama":{"baseUrl":"http://127.0.0.1:11434/v1","api":"openai-completions","authHeader":false,"models":[{"id":"ghost","name":"ghost","input":["text"],"reasoning":false,"contextWindow":8192,"maxTokens":2048}]}}}"#,
        )
        .expect("models.json");
        let auth = AuthStorage::load(pi::sdk::Config::auth_path()).expect("auth");
        let registry = ModelRegistry::load_for_listing(&auth, Some(path));
        let mut live = HashMap::new();
        live.insert("ollama".to_string(), Vec::new());
        let merged = merge_registry_with_live(&registry, &live);
        assert!(
            merged.iter().all(|row| !row.id.starts_with("ollama/")),
            "empty live ollama must hide models.json ghosts: {merged:?}"
        );
    });
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
fn kiss_bundled_pi_sdk_models_refresh_cache_tests() {
    provider_cache_round_trip_and_freshness();
    merge_registry_with_live_prefers_live_ids_and_enriches_metadata();
    refresh_skips_network_when_cache_is_fresh();
    merge_registry_with_live_keeps_static_when_provider_not_refreshed();
    empty_live_keyless_suppresses_static_ghost_rows();
    load_provider_cache_rejects_invalid_json();
    stale_cache_is_not_treated_as_fresh();
}
