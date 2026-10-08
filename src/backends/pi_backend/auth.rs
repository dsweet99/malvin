use crate::agent_process::AuthError;

use super::pi_backend::discover::resolve_npm_pi_entry;

pub fn ensure_npm_pi_authenticated(model: &str) -> Result<(), AuthError> {
    resolve_npm_pi_entry().map_err(AuthError)?;
    let parsed = crate::config::model_id::parse_model_id(model).map_err(AuthError)?;
    let Some((provider, _)) = parsed.pi_provider_and_model() else {
        return Err(AuthError(format!(
            "pi model id must be `pi:<provider>/<model>` (got `{model}`)"
        )));
    };
    if crate::backends::pi_backend::is_provider_authenticated(provider)
        || !crate::backends::pi_backend::provider_has_known_credentials(provider)
    {
        return Ok(());
    }
    Err(AuthError(format!(
        "pi backend is not authenticated for provider `{provider}`. {}.",
        crate::backends::pi_backend::missing_credentials_hint(provider)
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_fake_npm_pi_entry(body: impl FnOnce()) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let entry = tmp.path().join("rpc-entry.js");
        std::fs::write(&entry, "// fake npm pi entry\n").expect("write entry");
        crate::agent_process::with_env("MALVIN_PI", Some(entry.to_str().expect("utf8")), body);
    }
    #[test]
    fn rejects_when_entry_missing_and_allows_extension_providers_without_known_credentials() {
        {
            let _lock = crate::test_support::test_utils::test_env_lock();
            crate::agent_process::with_env("MALVIN_PI", Some("/missing/npm-pi.js"), || {
                let err = ensure_npm_pi_authenticated("pi:openai/gpt-4o").expect_err("missing");
                assert!(err.0.contains("MALVIN_PI") || err.0.contains("missing"));
            });
        }
        {
            let _lock = crate::test_support::test_utils::test_env_lock();
            with_fake_npm_pi_entry(|| {
                ensure_npm_pi_authenticated("pi:issue42-ext-provider/some-model")
                    .expect("extension providers must not be false-rejected by the auth gate");
            });
        }
    }

    #[test]
    fn still_rejects_known_provider_without_credentials() {
        let _lock = crate::test_support::test_utils::test_env_lock();
        with_fake_npm_pi_entry(|| {
            crate::agent_process::with_env("OPENAI_API_KEY", None, || {
                if crate::backends::pi_backend::is_provider_authenticated("openai") {
                    return;
                }
                let err = ensure_npm_pi_authenticated("pi:openai/gpt-4o")
                    .expect_err("known provider without credentials must still fail fast");
                assert!(err.0.contains("openai"));
                assert!(err.0.contains("not authenticated"));
            });
        });
    }

    #[test]
    fn keyless_local_models_pass_without_credentials() {
        let _lock = crate::test_support::test_utils::test_env_lock();
        with_fake_npm_pi_entry(|| {
            ensure_npm_pi_authenticated("pi:local/ollama/qwen2.5:1.5b").expect("local");
            ensure_npm_pi_authenticated("pi:llamacpp/m").expect("llamacpp");
        });
    }
}
