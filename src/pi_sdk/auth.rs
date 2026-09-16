use super::local_endpoint::{http_base_url_is_listening, keyless_local_provider_is_listening};
use crate::acp::AuthError;

pub fn ensure_pi_authenticated(model: &str) -> Result<(), AuthError> {
    let parsed = crate::model_id::parse_model_id(model).map_err(AuthError)?;
    let Some((provider, _)) = parsed.pi_provider_and_model() else {
        return Err(AuthError(format!(
            "rpi model id must be `rpi:<provider>/<model>` (got `{model}`)"
        )));
    };
    if provider_has_access(provider) {
        return Ok(());
    }
    provider_auth_env_keys(provider).map_or_else(
        || {
            Err(AuthError(format!(
                "pi backend is not authenticated for provider `{provider}`. Store credentials in Pi’s auth file ({}).",
                pi::sdk::Config::auth_path().display()
            )))
        },
        |keys| {
            Err(AuthError(format!(
                "pi backend is not authenticated for provider `{provider}`. Set {} or store credentials in Pi’s auth file ({}).",
                keys.join(" or "),
                pi::sdk::Config::auth_path().display()
            )))
        },
    )
}

pub fn is_provider_authenticated(provider: &str) -> bool {
    provider_has_access(provider)
}

#[must_use]
pub fn provider_known_in_rust_metadata(provider: &str) -> bool {
    pi::provider_metadata::canonical_provider_id(provider).is_some()
}

pub fn is_provider_listable(provider: &str) -> bool {
    if pi::provider_metadata::provider_is_keyless_local(provider) {
        return keyless_local_provider_is_listening(provider);
    }
    if let Some(custom) = models_json_provider(provider)
        && !custom.auth_header
    {
        return custom
            .base_url
            .as_deref()
            .is_some_and(http_base_url_is_listening);
    }
    provider_has_stored_or_env_access(provider)
}

fn provider_has_access(provider: &str) -> bool {
    if pi::provider_metadata::provider_is_keyless_local(provider) {
        return true;
    }
    if custom_provider_is_keyless(provider) {
        return true;
    }
    provider_has_stored_or_env_access(provider)
}

fn provider_has_stored_or_env_access(provider: &str) -> bool {
    match provider_auth_env_keys(provider) {
        None => stored_credential_present(provider),
        Some(keys) if keys.iter().any(|k| crate::acp::env_key_nonempty(k)) => true,
        Some(_) => stored_credential_present(provider),
    }
}

struct ModelsJsonProvider {
    auth_header: bool,
    base_url: Option<String>,
}

fn models_json_provider(provider: &str) -> Option<ModelsJsonProvider> {
    let models_path = pi::models::default_models_path(&pi::sdk::Config::global_dir());
    let body = std::fs::read_to_string(models_path).ok()?;
    let root: serde_json::Value = serde_json::from_str(&body).ok()?;
    let providers = root.get("providers")?.as_object()?;
    for (name, cfg) in providers {
        if !pi::provider_metadata::provider_ids_match(name, provider) {
            continue;
        }
        let auth_header = cfg
            .get("authHeader")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(true);
        let base_url = cfg
            .get("baseUrl")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|url| !url.is_empty())
            .map(str::to_string);
        return Some(ModelsJsonProvider {
            auth_header,
            base_url,
        });
    }
    None
}

fn custom_provider_is_keyless(provider: &str) -> bool {
    models_json_provider(provider).is_some_and(|entry| !entry.auth_header)
}

fn stored_credential_present(provider: &str) -> bool {
    let Ok(auth) = pi::auth::AuthStorage::load(pi::sdk::Config::auth_path()) else {
        return false;
    };
    !matches!(
        auth.credential_status(provider),
        pi::auth::CredentialStatus::Missing
    ) || auth.has_stored_credential(provider)
}

#[must_use]
pub(crate) fn provider_auth_env_keys(provider: &str) -> Option<&'static [&'static str]> {
    let keys = pi::provider_metadata::provider_auth_env_keys(provider);
    if keys.is_empty() { None } else { Some(keys) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapped_provider_requires_key() {
        crate::acp::with_env("OPENAI_API_KEY", None, || {
            if !stored_credential_present("openai") {
                assert!(ensure_pi_authenticated("rpi:openai/gpt-4o").is_err());
            }
        });
    }

    #[test]
    fn keyless_local_providers_skip_credential_gate() {
        for provider in ["ollama", "llamacpp", "mistralrs"] {
            assert!(
                is_provider_authenticated(provider),
                "{provider} must be runnable without stored credentials"
            );
            assert!(
                ensure_pi_authenticated(&format!("rpi:{provider}/some-model")).is_ok(),
                "{provider} must pass ensure_pi_authenticated"
            );
        }
        assert!(ensure_pi_authenticated("rpi:local/whatever_model_name").is_ok());
        assert!(ensure_pi_authenticated("rpi:local/ollama/qwen2.5:1.5b").is_ok());
        assert!(!pi::provider_metadata::provider_is_keyless_local(
            "lmstudio"
        ));
        for provider in ["llamacpp", "mistralrs"] {
            if !keyless_local_provider_is_listening(provider) {
                assert!(
                    !is_provider_listable(provider),
                    "{provider} must be hidden from admin models when its server is down"
                );
            }
        }
    }

    #[test]
    fn models_json_does_not_unlock_headerless_cloud_providers() {
        let _lock = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().expect("tempdir");
        let home = tmp.path().join("pi-home");
        std::fs::create_dir_all(&home).expect("pi home");
        std::fs::write(
            home.join("models.json"),
            r#"{"providers":{"ollama":{"baseUrl":"http://127.0.0.1:11434/v1","api":"openai-completions","authHeader":false,"models":[{"id":"lite"}]}}}"#,
        )
        .expect("write models.json");
        crate::acp::with_env(
            "PI_CODING_AGENT_DIR",
            Some(home.to_str().expect("utf8")),
            || {
                crate::acp::with_env("ZENMUX_API_KEY", None, || {
                    crate::acp::with_env("ANTHROPIC_API_KEY", None, || {
                        crate::acp::with_env("COHERE_API_KEY", None, || {
                            assert!(
                                custom_provider_is_keyless("ollama"),
                                "ollama models.json entry is keyless"
                            );
                            assert!(!custom_provider_is_keyless("zenmux"));
                            assert!(!custom_provider_is_keyless("anthropic"));
                            assert!(!custom_provider_is_keyless("cohere"));
                            assert!(!custom_provider_is_keyless("google"));
                            assert!(!custom_provider_is_keyless("amazon-bedrock"));
                            assert!(!is_provider_listable("zenmux"));
                            assert!(!is_provider_listable("anthropic"));
                            assert!(!is_provider_listable("cohere"));
                        });
                    });
                });
            },
        );
    }

    #[test]
    fn unknown_provider_requires_stored_credential() {
        crate::acp::with_env("OPENAI_API_KEY", None, || {
            assert!(!is_provider_authenticated("some-unknown"));
            let err = ensure_pi_authenticated("rpi:some-unknown/foo").expect_err("must fail");
            assert!(err.0.contains("some-unknown"));
        });
    }

    #[test]
    fn finish_after_channel_closed_maps_dropped_reply_to_error() {
        let err = crate::pi_sdk::session::finish_after_channel_closed(Err(
            "pi sdk runtime stopped".into(),
        ))
        .expect_err("dropped reply must fail");
        assert!(err.message.contains("runtime stopped"));
    }

    #[test]
    fn is_provider_authenticated_checks_known_and_unknown_providers() {
        crate::acp::with_env("OPENAI_API_KEY", None, || {
            if !stored_credential_present("openai") {
                assert!(!is_provider_authenticated("openai"));
            }
        });
        crate::acp::with_env("OPENAI_API_KEY", Some("test-key"), || {
            assert!(is_provider_authenticated("openai"));
        });
        assert!(!is_provider_authenticated("some-unknown"));
    }

    #[test]
    fn is_provider_authenticated_checks_primary_and_secondary() {
        crate::acp::with_env("DEEPSEEK_API_KEY", Some("test-key"), || {
            assert!(is_provider_authenticated("deepseek"));
            assert!(is_provider_authenticated("deep-seek"));
        });
        crate::acp::with_env("DEEPSEEK_API_KEY", None, || {
            if !stored_credential_present("deepseek") && !stored_credential_present("deep-seek") {
                assert!(!is_provider_authenticated("deepseek"));
            }
        });
    }
}
