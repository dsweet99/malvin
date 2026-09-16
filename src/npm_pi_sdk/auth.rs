use crate::acp::AuthError;

use super::discover::resolve_npm_pi_entry;

pub fn ensure_npm_pi_authenticated(model: &str) -> Result<(), AuthError> {
    resolve_npm_pi_entry().map_err(AuthError)?;
    let parsed = crate::model_id::parse_model_id(model).map_err(AuthError)?;
    let Some((provider, _)) = parsed.pi_provider_and_model() else {
        return Err(AuthError(format!(
            "pi model id must be `pi:<provider>/<model>` (got `{model}`)"
        )));
    };
    if crate::pi_sdk::is_provider_authenticated(provider) {
        return Ok(());
    }
    if !crate::pi_sdk::provider_known_in_rust_metadata(provider) {
        return Ok(());
    }
    Err(AuthError(format!(
        "pi backend is not authenticated for provider `{provider}`. Set a provider API key or store credentials for the npm Pi agent."
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_fake_npm_pi_entry(body: impl FnOnce()) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let entry = tmp.path().join("rpc-entry.js");
        std::fs::write(&entry, "// fake npm pi entry\n").expect("write entry");
        crate::acp::with_env(
            "MALVIN_PI",
            Some(entry.to_str().expect("utf8")),
            body,
        );
    }

    #[test]
    fn rejects_when_entry_missing() {
        let _lock = crate::test_utils::test_env_lock();
        crate::acp::with_env("MALVIN_PI", Some("/missing/npm-pi.js"), || {
            let err = ensure_npm_pi_authenticated("pi:openai/gpt-4o").expect_err("missing");
            assert!(err.0.contains("MALVIN_PI") || err.0.contains("missing"));
        });
    }

    #[test]
    fn allows_extension_providers_without_rust_auth_metadata() {
        let _lock = crate::test_utils::test_env_lock();
        with_fake_npm_pi_entry(|| {
            ensure_npm_pi_authenticated("pi:issue42-ext-provider/some-model")
                .expect("extension providers must not be false-rejected by rust-Pi auth gate");
        });
    }

    #[test]
    fn still_rejects_known_provider_without_credentials() {
        let _lock = crate::test_utils::test_env_lock();
        with_fake_npm_pi_entry(|| {
            crate::acp::with_env("OPENAI_API_KEY", None, || {
                if crate::pi_sdk::is_provider_authenticated("openai") {
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
    fn still_rejects_empty_env_key_builtins_without_credentials() {
        let _lock = crate::test_utils::test_env_lock();
        with_fake_npm_pi_entry(|| {
            assert!(crate::pi_sdk::provider_known_in_rust_metadata("openai-codex"));
            if crate::pi_sdk::is_provider_authenticated("openai-codex") {
                return;
            }
            let err = ensure_npm_pi_authenticated("pi:openai-codex/gpt-5")
                .expect_err("known empty-env-key builtins must still fail fast");
            assert!(err.0.contains("openai-codex"));
        });
    }
}
