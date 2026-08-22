use std::collections::HashMap;

use pi::auth::AuthStorage;
use pi::sdk::{Config, ModelRegistry};

use super::auth::env_nonempty;

fn env_keys_for_provider(provider: &str) -> Vec<String> {
    super::auth::provider_auth_env_keys(provider)
        .map(|keys| keys.iter().map(ToString::to_string).collect())
        .unwrap_or_default()
}

pub fn list_pi_provider_auth_sync() -> Result<HashMap<String, Vec<String>>, String> {
    let auth = AuthStorage::load(Config::auth_path()).map_err(|e| e.to_string())?;
    let registry = ModelRegistry::load_for_listing(&auth, None);
    let mut map = HashMap::new();
    for entry in registry.models() {
        let provider = entry.model.provider.as_str();
        let keys = env_keys_for_provider(provider);
        map.entry(provider.to_string()).or_insert(keys);
    }
    if map.is_empty() {
        return Err("pi model registry produced no providers".to_string());
    }
    Ok(map)
}

#[must_use]
pub fn provider_authenticated_from_map(provider: &str, map: &HashMap<String, Vec<String>>) -> bool {
    map.get(provider)
        .is_some_and(|keys| !keys.is_empty() && keys.iter().any(|k| env_nonempty(k)))
}

#[cfg(test)]
mod providers_list_tests {
    use std::collections::HashMap;

    use super::{list_pi_provider_auth_sync, provider_authenticated_from_map};

    #[test]
    fn provider_authenticated_from_map_honors_env_and_unknown() {
        let mut map = HashMap::new();
        map.insert("openai".into(), vec!["OPENAI_API_KEY".into()]);
        map.insert("llamacpp".into(), vec![]);
        crate::acp::with_env("OPENAI_API_KEY", None, || {
            assert!(!provider_authenticated_from_map("openai", &map));
            assert!(!provider_authenticated_from_map("llamacpp", &map));
            assert!(!provider_authenticated_from_map("unknown", &map));
        });
        crate::acp::with_env("OPENAI_API_KEY", Some(""), || {
            assert!(!provider_authenticated_from_map("openai", &map));
        });
        crate::acp::with_env("OPENAI_API_KEY", Some("   "), || {
            assert!(!provider_authenticated_from_map("openai", &map));
        });
        crate::acp::with_env("OPENAI_API_KEY", Some("k"), || {
            assert!(provider_authenticated_from_map("openai", &map));
        });
    }

    #[test]
    fn list_pi_provider_auth_sync_reads_crate_registry() {
        use crate::test_utils::test_env_lock;

        let _lock = test_env_lock();
        let tmp = tempfile::tempdir().expect("tmpdir");
        crate::acp::with_env(
            "PI_CODING_AGENT_DIR",
            Some(tmp.path().to_str().expect("utf8")),
            || {
                let map = list_pi_provider_auth_sync().expect("crate providers");
                assert!(map.contains_key("openai"));
                assert!(map.contains_key("openrouter"));
            },
        );
    }
}
