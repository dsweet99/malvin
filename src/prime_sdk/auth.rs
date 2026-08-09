//! Auth for the Prime SDK backend (provider keys via Prime `AuthStorage` in the bridge).

use crate::acp::AuthError;

/// Common provider API key env vars Prime may read (never Cursor keys).
#[must_use]
pub fn effective_prime_api_key() -> Option<String> {
    for key in [
        "OPENAI_API_KEY",
        "ANTHROPIC_API_KEY",
        "OPENROUTER_API_KEY",
        "PRIME_API_KEY",
    ] {
        if let Ok(v) = std::env::var(key) {
            let t = v.trim().trim_start_matches('\u{feff}');
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

/// Soft check: at least one provider key is present. The bridge uses Prime `AuthStorage`.
///
/// `prime:local/…` uses a malvin GGUF sidecar and does not need cloud keys.
///
/// # Errors
///
/// Returns [`AuthError`] when no known provider API key is set (and the model is not local).
pub fn ensure_prime_authenticated(model: &str) -> Result<(), AuthError> {
    if crate::model_id::uses_prime_local_backend(model) {
        return Ok(());
    }
    if effective_prime_api_key().is_some() {
        return Ok(());
    }
    Err(AuthError(
        "Prime SDK backend is not authenticated. Set a provider API key (OPENAI_API_KEY, ANTHROPIC_API_KEY, OPENROUTER_API_KEY, or PRIME_API_KEY). Never use Cursor credentials for prime: models."
            .to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_prime_authenticated_ok_with_key() {
        crate::acp::with_env("OPENAI_API_KEY", Some("test-key"), || {
            assert!(ensure_prime_authenticated("prime:openai/gpt-4o").is_ok());
        });
    }

    #[test]
    fn prime_local_skips_cloud_api_key() {
        crate::acp::with_env("OPENAI_API_KEY", None, || {
            crate::acp::with_env("ANTHROPIC_API_KEY", None, || {
                crate::acp::with_env("OPENROUTER_API_KEY", None, || {
                    crate::acp::with_env("PRIME_API_KEY", None, || {
                        assert!(ensure_prime_authenticated(
                            "prime:local/qwen35_9b_q4"
                        )
                        .is_ok());
                    });
                });
            });
        });
    }

    #[test]
    fn never_treats_cursor_key_as_prime_auth() {
        crate::acp::with_env("OPENAI_API_KEY", None, || {
            crate::acp::with_env("ANTHROPIC_API_KEY", None, || {
                crate::acp::with_env("OPENROUTER_API_KEY", None, || {
                    crate::acp::with_env("PRIME_API_KEY", None, || {
                        crate::acp::with_env("CURSOR_API_KEY", Some("cursor-only"), || {
                            assert!(ensure_prime_authenticated("prime:openai/gpt-4o").is_err());
                        });
                    });
                });
            });
        });
    }
}
