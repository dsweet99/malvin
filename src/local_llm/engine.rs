//! Load a cached GGUF into an in-process llama.cpp engine.

use std::sync::Arc;

use crate::malvin_llama::{
    complete as llama_complete, load_engine_with_context_size, ChatTurn, CompleteRequest,
    LocalEngine,
};
use crate::llm_transport::{
    ChatMessage, ChatRole, CompletionResponse, HttpExchangeMeta, TransportError,
};

use super::download::{ensure_model_cached, DownloadPolicy};
use super::registry::{require_known_local_slug, LocalModelSpec};
use crate::model_id::{parse_model_id, LOCAL_PREFIX, ModelBackend};

const DEFAULT_MAX_TOKENS: i32 = 2048;

enum LocalEngineInner {
    Real(Arc<LocalEngine>),
    #[cfg(test)]
    Scripted(Result<String, String>),
}

/// In-process local completion backend for the mini loop.
pub struct LocalCompletionEngine {
    inner: LocalEngineInner,
    pub model_slug: String,
}

impl LocalCompletionEngine {
    #[cfg(test)]
    #[must_use]
    pub fn scripted_ok(model_slug: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            inner: LocalEngineInner::Scripted(Ok(content.into())),
            model_slug: model_slug.into(),
        }
    }

    #[cfg(test)]
    #[must_use]
    pub fn scripted_err(model_slug: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            inner: LocalEngineInner::Scripted(Err(detail.into())),
            model_slug: model_slug.into(),
        }
    }

    /// # Errors
    ///
    /// Returns an OpenRouter-shaped error when generation fails.
    pub async fn complete(
        &self,
        messages: &[ChatMessage],
    ) -> (
        Result<CompletionResponse, TransportError>,
        HttpExchangeMeta,
    ) {
        match &self.inner {
            LocalEngineInner::Real(engine) => {
                let turns = messages_to_turns(messages);
                let max_tokens = local_max_tokens_from_env();
                let engine = Arc::clone(engine);
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
            #[cfg(test)]
            LocalEngineInner::Scripted(scripted) => {
                let _ = messages;
                map_complete_result(scripted.clone())
            }
        }
    }
}

pub(super) fn map_complete_result(
    result: Result<String, String>,
) -> (
    Result<CompletionResponse, TransportError>,
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
            Err(TransportError::Engine(e)),
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
        inner: LocalEngineInner::Real(Arc::new(engine)),
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
#[path = "engine_tests.rs"]
mod engine_tests;
