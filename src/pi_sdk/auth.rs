use crate::acp::AuthError;

pub fn ensure_pi_authenticated(model: &str) -> Result<(), AuthError> {
    let parsed = crate::model_id::parse_model_id(model).map_err(AuthError)?;
    let Some((provider, _)) = parsed.pi_provider_and_model() else {
        return Err(AuthError(format!(
            "pi model id must be `pi:<provider>/<model>` (got `{model}`)"
        )));
    };
    if provider_has_access(provider) {
        return Ok(());
    }
    let keys = provider_auth_env_keys(provider).unwrap_or(&[]);
    if keys.is_empty() {
        return Ok(());
    }
    Err(AuthError(format!(
        "pi backend is not authenticated for provider `{provider}`. Set {} or store credentials in Pi’s auth file ({}).",
        keys.join(" or "),
        pi::sdk::Config::auth_path().display()
    )))
}

pub fn is_provider_authenticated(provider: &str) -> bool {
    provider_has_access(provider)
}

fn provider_has_access(provider: &str) -> bool {
    if provider_auth_env_keys(provider).is_none_or(|keys| keys.iter().any(|k| env_nonempty(k))) {
        return true;
    }
    stored_credential_present(provider)
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
    provider_auth_env_keys_primary(provider).or_else(|| provider_auth_env_keys_secondary(provider))
}

fn provider_auth_env_keys_primary(provider: &str) -> Option<&'static [&'static str]> {
    match provider {
        "openai" => Some(&["OPENAI_API_KEY"]),
        "anthropic" => Some(&["ANTHROPIC_API_KEY"]),
        "openrouter" => Some(&["OPENROUTER_API_KEY"]),
        "google" | "gemini" => Some(&["GOOGLE_API_KEY", "GEMINI_API_KEY"]),
        "groq" => Some(&["GROQ_API_KEY"]),
        _ => None,
    }
}

fn provider_auth_env_keys_secondary(provider: &str) -> Option<&'static [&'static str]> {
    match provider {
        "deepseek" | "deep-seek" => Some(&["DEEPSEEK_API_KEY"]),
        "cerebras" => Some(&["CEREBRAS_API_KEY"]),
        "xai" => Some(&["XAI_API_KEY"]),
        "mistral" => Some(&["MISTRAL_API_KEY"]),
        _ => None,
    }
}

pub(crate) fn env_nonempty(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|v| !v.trim().trim_start_matches('\u{feff}').is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapped_provider_requires_key() {
        crate::acp::with_env("OPENAI_API_KEY", None, || {
            if !stored_credential_present("openai") {
                assert!(ensure_pi_authenticated("pi:openai/gpt-4o").is_err());
            }
        });
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
        assert!(is_provider_authenticated("some-unknown"));
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
