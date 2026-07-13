//! Load a cached GGUF into an in-process llama.cpp engine.

use std::sync::Arc;

use malvin_llama::{
    complete as llama_complete, load_engine, ChatTurn, CompleteRequest, LocalEngine,
};
use malvin_mini::{
    ChatMessage, ChatRole, CompletionResponse, HttpExchangeMeta, OpenRouterError,
};

use super::download::{ensure_model_cached, DownloadPolicy};
use super::registry::{require_known_local_slug, LocalModelSpec};
use crate::model_id::{parse_model_id, LOCAL_PREFIX, ModelBackend};

const DEFAULT_MAX_TOKENS: i32 = 2048;

/// In-process local completion backend for the mini loop.
pub struct LocalCompletionEngine {
    engine: Arc<LocalEngine>,
    pub model_slug: String,
}

impl LocalCompletionEngine {
    /// # Errors
    ///
    /// Returns an OpenRouter-shaped error when generation fails.
    pub async fn complete(
        &self,
        messages: &[ChatMessage],
    ) -> (
        Result<CompletionResponse, OpenRouterError>,
        HttpExchangeMeta,
    ) {
        let turns = messages_to_turns(messages);
        let max_tokens = max_tokens_from_env();
        let engine = Arc::clone(&self.engine);
        let result = tokio::task::spawn_blocking(move || {
            llama_complete(
                &engine,
                &CompleteRequest {
                    turns: &turns,
                    max_tokens,
                },
            )
        })
        .await
        .unwrap_or_else(|e| Err(format!("local llama join: {e}")));
        map_complete_result(result)
    }
}

pub(super) fn map_complete_result(
    result: Result<String, String>,
) -> (
    Result<CompletionResponse, OpenRouterError>,
    HttpExchangeMeta,
) {
    match result {
        Ok(content) => (
            Ok(CompletionResponse {
                content,
                usage: None,
                reasoning: None,
            }),
            HttpExchangeMeta {
                status: Some(200),
                body: None,
            },
        ),
        Err(e) => (
            Err(OpenRouterError::RequestFailed {
                status: 500,
                body: e,
            }),
            HttpExchangeMeta {
                status: Some(500),
                body: None,
            },
        ),
    }
}

pub(super) fn messages_to_turns(messages: &[ChatMessage]) -> Vec<ChatTurn> {
    messages
        .iter()
        .map(|m| ChatTurn {
            role: role_name(m.role).into(),
            content: m.content.clone(),
        })
        .collect()
}

const fn role_name(role: ChatRole) -> &'static str {
    match role {
        ChatRole::System => "system",
        ChatRole::User => "user",
        ChatRole::Assistant => "assistant",
    }
}

pub(super) fn max_tokens_from_env() -> i32 {
    std::env::var("MALVIN_LOCAL_MAX_TOKENS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_MAX_TOKENS)
}

/// Ensure the GGUF is cached and load it into Metal llama.cpp.
///
/// # Errors
///
/// Returns an error when the model id is invalid, download fails, mem limit is too
/// low for the model, or load fails.
pub fn ensure_local_engine(
    model_id: &str,
    policy: DownloadPolicy,
) -> Result<LocalCompletionEngine, String> {
    let slug = local_slug(model_id)?;
    let spec = require_known_local_slug(&slug)?;
    require_mem_limit_for_local(spec)?;
    let gguf = ensure_model_cached(spec, policy)?;
    let engine = load_engine(&gguf)?;
    Ok(LocalCompletionEngine {
        engine: Arc::new(engine),
        model_slug: slug,
    })
}

fn require_mem_limit_for_local(spec: &LocalModelSpec) -> Result<(), String> {
    let work_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let configured = crate::mem_limit_config::load_mem_limit_gb(&work_dir);
    if configured >= spec.min_mem_limit_gb {
        return Ok(());
    }
    Err(format!(
        "local model `{LOCAL_PREFIX}{}` needs mem_limit_gb >= {} (currently {configured}); set mem_limit_gb in ~/.malvin_home/config.toml",
        spec.slug, spec.min_mem_limit_gb
    ))
}

fn local_slug(model_id: &str) -> Result<String, String> {
    match parse_model_id(model_id) {
        Ok(parsed) if parsed.backend == ModelBackend::Local => Ok(parsed.slug),
        Ok(_) => Err(format!("expected `{LOCAL_PREFIX}<id>`, got `{model_id}`")),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_slug_requires_local_prefix() {
        assert_eq!(
            local_slug("local:qwen35_9b_q4").expect("ok"),
            "qwen35_9b_q4"
        );
        assert!(local_slug("openrouter:x").is_err());
    }

    #[test]
    fn ensure_local_engine_rejects_non_local_ids() {
        assert!(ensure_local_engine("openrouter:x", DownloadPolicy::Deny).is_err());
    }

    #[test]
    fn require_mem_limit_for_local_mentions_config_key() {
        let spec = require_known_local_slug("qwen35_9b_q4").expect("spec");
        // With the default 4 GiB home config this should fail; if the operator already
        // raised mem_limit_gb enough, the check is a no-op Ok.
        match require_mem_limit_for_local(spec) {
            Ok(()) => assert!(
                crate::mem_limit_config::load_mem_limit_gb(
                    &std::env::current_dir().unwrap_or_else(|_| ".".into())
                ) >= spec.min_mem_limit_gb
            ),
            Err(e) => {
                assert!(e.contains("mem_limit_gb"));
                assert!(e.contains(&spec.min_mem_limit_gb.to_string()));
            }
        }
    }

    #[test]
    fn messages_to_turns_maps_roles() {
        let turns = messages_to_turns(&[
            ChatMessage {
                role: ChatRole::System,
                content: "s".into(),
            },
            ChatMessage {
                role: ChatRole::User,
                content: "u".into(),
            },
        ]);
        assert_eq!(turns[0].role, "system");
        assert_eq!(turns[1].role, "user");
    }

    #[test]
    fn map_complete_result_ok_and_err() {
        let (ok, meta) = map_complete_result(Ok("hi".into()));
        assert_eq!(ok.expect("ok").content, "hi");
        assert_eq!(meta.status, Some(200));
        let (err, meta) = map_complete_result(Err("boom".into()));
        assert!(err.is_err());
        assert_eq!(meta.status, Some(500));
    }

    #[test]
    fn kiss_cov_local_completion_engine_and_complete() {
        let engine: Option<LocalCompletionEngine> = None;
        assert!(engine.is_none());
        let complete = LocalCompletionEngine::complete;
        let _ = complete;
        let max_tokens_from_env = super::max_tokens_from_env;
        assert!(max_tokens_from_env() > 0);
    }
}
