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
    Err(AuthError(format!(
        "pi backend is not authenticated for provider `{provider}`. Set a provider API key or store credentials for the npm Pi agent."
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_when_entry_missing() {
        let _lock = crate::test_utils::test_env_lock();
        crate::acp::with_env("MALVIN_PI", Some("/missing/npm-pi.js"), || {
            let err = ensure_npm_pi_authenticated("pi:openai/gpt-4o").expect_err("missing");
            assert!(err.0.contains("MALVIN_PI") || err.0.contains("missing"));
        });
    }
}
