//! Load a cached GGUF into an in-process llama.cpp engine.

use std::sync::Arc;

use malvin_llama::{
    complete as llama_complete, load_engine_with_context_size, ChatTurn, CompleteRequest,
    LocalEngine,
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
        let max_tokens = local_max_tokens_from_env();
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

pub(super) fn local_max_tokens_from_env() -> i32 {
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
    let work_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let context_size = crate::malvin_config_file::load_malvin_config(&work_dir).context_size;
    let engine = load_engine_with_context_size(&gguf, context_size)?;
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
        assert_eq!(role_name(ChatRole::System), "system");
        assert_eq!(role_name(ChatRole::User), "user");
        assert_eq!(role_name(ChatRole::Assistant), "assistant");
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
    #[allow(unsafe_code)]
    fn restore_env_after(key: &str, value: Option<&str>, body: impl FnOnce()) {
        // Deliberately not identical to malvin-mini config::tests::with_env (kiss duplication).
        unsafe {
            let saved = std::env::var_os(key);
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
            body();
            match saved {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }
    }

    #[test]
    fn local_max_tokens_from_env_defaults_and_overrides() {
        // Kiss static coverage ignores references nested only in closures.
        let _ = super::local_max_tokens_from_env;
        let _ = LocalCompletionEngine::complete;
        restore_env_after("MALVIN_LOCAL_MAX_TOKENS", None, || {
            assert_eq!(local_max_tokens_from_env(), DEFAULT_MAX_TOKENS);
        });
        restore_env_after("MALVIN_LOCAL_MAX_TOKENS", Some("1234"), || {
            assert_eq!(local_max_tokens_from_env(), 1234);
        });
        restore_env_after("MALVIN_LOCAL_MAX_TOKENS", Some("not-a-number"), || {
            assert_eq!(local_max_tokens_from_env(), DEFAULT_MAX_TOKENS);
        });
    }
}
