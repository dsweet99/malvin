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

fn provider_has_access(provider: &str) -> bool {
    if pi::provider_metadata::provider_is_keyless_local(provider) {
        return true;
    }
    if custom_provider_is_keyless(provider) {
        return true;
    }
    match provider_auth_env_keys(provider) {
        None => stored_credential_present(provider),
        Some(keys) if keys.iter().any(|k| crate::acp::env_key_nonempty(k)) => true,
        Some(_) => stored_credential_present(provider),
    }
}

fn custom_provider_is_keyless(provider: &str) -> bool {
    let Ok(auth) = pi::auth::AuthStorage::load(pi::sdk::Config::auth_path()) else {
        return false;
    };
    let models_path = pi::models::default_models_path(&pi::sdk::Config::global_dir());
    if !models_path.is_file() {
        return false;
    }
    let registry = pi::sdk::ModelRegistry::load_for_listing(&auth, Some(models_path));
    registry.models().iter().any(|entry| {
        pi::provider_metadata::provider_ids_match(&entry.model.provider, provider)
            && !entry.auth_header
    })
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
        assert!(!pi::provider_metadata::provider_is_keyless_local("lmstudio"));
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
