//! Construct Mini [`LlmBackend`] from model id and download policy.

use super::LlmBackend;
use crate::llm_transport::{LocalLlmTransport, LlmTransport, OpenRouterTransport};

pub fn build_llm_backend(model: &str, allow_download: bool) -> Result<LlmBackend, String> {
    if crate::model_id::uses_local_backend(model) {
        let policy = if allow_download {
            crate::local_llm::DownloadPolicy::Allow
        } else {
            crate::local_llm::DownloadPolicy::Deny
        };
        let engine = crate::local_llm::ensure_local_engine(model, policy)?;
        Ok(LlmBackend::Transport(LlmTransport::Local(LocalLlmTransport::new(
            engine,
        ))))
    } else if crate::model_id::uses_openrouter_backend(model) {
        let openrouter_config = crate::openrouter_transport::OpenRouterConfig::from_env(
            super::resolve_mini_model(model),
        )?;
        let transport = OpenRouterTransport::new(openrouter_config)
            .map_err(|e| format!("OpenRouter client init failed: {e}"))?;
        Ok(LlmBackend::Transport(LlmTransport::OpenRouter(transport)))
    } else {
        Err(format!(
            "build_llm_backend expects `mini:openrouter/…` or `mini:local/…` (got `{model}`)"
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::build_llm_backend;
    use crate::llm_transport::LlmTransport;
    use crate::mini_agent::LlmBackend;

    #[allow(unsafe_code)]
    #[test]
    fn build_llm_backend_selects_openrouter_by_prefix() {
        unsafe {
            std::env::set_var("OPENROUTER_API_KEY", "sk-test-build-llm-backend");
        }
        let backend =
            build_llm_backend("mini:openrouter/openai/gpt-4o", false).expect("openrouter");
        assert!(matches!(
            backend,
            LlmBackend::Transport(LlmTransport::OpenRouter(_))
        ));
    }

    #[test]
    fn build_llm_backend_selects_local_path_by_prefix() {
        let Err(err) = build_llm_backend("mini:local/not_a_known_local_slug", false) else {
            panic!("expected local unknown slug to fail");
        };
        assert!(err.contains("unknown local model"), "{err}");
    }

    #[test]
    fn build_llm_backend_rejects_non_mini() {
        let err = build_llm_backend("cursor:auto", false).err().expect("non-mini");
        assert!(err.contains("mini:"));
        let err = build_llm_backend("prime:openai/gpt-5.5", false)
            .err()
            .expect("prime");
        assert!(err.contains("mini:"));
    }
}
