use serde_json::Value;

use crate::local_llm::provider_metadata::{
    pi_auth_json_path, pi_models_json_path, provider_auth_env_keys, provider_ids_match,
    provider_is_keyless_local,
};

fn read_json(path: &std::path::Path) -> Option<Value> {
    let body = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&body).ok()
}

fn models_json_defines_provider(provider: &str) -> bool {
    read_json(&pi_models_json_path())
        .as_ref()
        .and_then(|root| root.get("providers"))
        .and_then(Value::as_object)
        .is_some_and(|providers| {
            providers
                .keys()
                .any(|name| provider_ids_match(name, provider))
        })
}

fn stored_credential(provider: &str) -> Option<Value> {
    let root = read_json(&pi_auth_json_path())?;
    root.as_object()?
        .iter()
        .find(|(name, _)| provider_ids_match(name, provider))
        .map(|(_, cred)| cred.clone())
}

fn env_api_key(provider: &str) -> Option<String> {
    provider_auth_env_keys(provider)?
        .iter()
        .find_map(|key| std::env::var(key).ok().filter(|v| !v.trim().is_empty()))
}

#[must_use]
pub(crate) fn provider_api_key(provider: &str) -> Option<String> {
    env_api_key(provider).or_else(|| {
        let cred = stored_credential(provider)?;
        let key = cred.get("key").and_then(Value::as_str)?.trim();
        (!key.is_empty() && !key.starts_with('!')).then(|| key.to_string())
    })
}

#[must_use]
pub fn is_provider_authenticated(provider: &str) -> bool {
    provider_is_keyless_local(provider)
        || models_json_defines_provider(provider)
        || env_api_key(provider).is_some()
        || stored_credential(provider).is_some()
}

#[must_use]
pub fn provider_has_known_credentials(provider: &str) -> bool {
    provider_auth_env_keys(provider).is_some()
}

#[must_use]
pub(crate) fn missing_credentials_hint(provider: &str) -> String {
    let auth = pi_auth_json_path();
    provider_auth_env_keys(provider).map_or_else(
        || format!("store credentials in Pi’s auth file ({})", auth.display()),
        |keys| {
            format!(
                "set {} or store credentials in Pi’s auth file ({})",
                keys.join(" or "),
                auth.display()
            )
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_pi_dir(models_json: Option<&str>, auth_json: Option<&str>, body: impl FnOnce()) {
        let _lock = crate::test_support::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().expect("tempdir");
        if let Some(text) = models_json {
            std::fs::write(tmp.path().join("models.json"), text).expect("models.json");
        }
        if let Some(text) = auth_json {
            std::fs::write(tmp.path().join("auth.json"), text).expect("auth.json");
        }
        crate::agent_process::with_env(
            "PI_CODING_AGENT_DIR",
            Some(tmp.path().to_str().expect("utf8")),
            body,
        );
    }

    #[test]
    fn keyless_local_providers_skip_credential_gate() {
        with_pi_dir(None, None, || {
            for provider in ["ollama", "llamacpp", "mistralrs"] {
                assert!(is_provider_authenticated(provider), "{provider}");
            }
        });
    }

    #[test]
    fn env_key_or_stored_credential_authenticates() {
        with_pi_dir(
            None,
            Some(r#"{"deepseek":{"type":"api_key","key":"sk-x"}}"#),
            || {
                crate::agent_process::with_env("OPENAI_API_KEY", None, || {
                    assert!(!is_provider_authenticated("openai"));
                });
                crate::agent_process::with_env("OPENAI_API_KEY", Some("test-key"), || {
                    assert!(is_provider_authenticated("openai"));
                    assert_eq!(provider_api_key("openai").as_deref(), Some("test-key"));
                });
                crate::agent_process::with_env("DEEPSEEK_API_KEY", None, || {
                    assert!(is_provider_authenticated("deepseek"));
                    assert_eq!(provider_api_key("deepseek").as_deref(), Some("sk-x"));
                });
            },
        );
    }

    #[test]
    fn command_credentials_are_not_returned_as_keys() {
        with_pi_dir(
            None,
            Some(r#"{"openrouter":{"type":"api_key","key":"!pass x"}}"#),
            || {
                crate::agent_process::with_env("OPENROUTER_API_KEY", None, || {
                    assert!(is_provider_authenticated("openrouter"));
                    assert!(provider_api_key("openrouter").is_none());
                });
            },
        );
    }

    #[test]
    fn models_json_providers_count_as_configured() {
        let models = r#"{"providers":{"portkey":{"baseUrl":"https://api.portkey.ai/v1","api":"openai-completions","models":[]}}}"#;
        with_pi_dir(Some(models), None, || {
            assert!(is_provider_authenticated("portkey"));
            assert!(!is_provider_authenticated("some-unknown"));
            assert!(!provider_has_known_credentials("portkey"));
            assert!(provider_has_known_credentials("openai"));
            assert!(missing_credentials_hint("openai").contains("OPENAI_API_KEY"));
        });
    }
}
