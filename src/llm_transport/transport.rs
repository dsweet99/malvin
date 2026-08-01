//! Neutral LLM transport surface (Mini → `OpenRouter` or Local).
//!
//! Implemented as an owning enum (kiss forbids trait definitions in this tree).

use crate::llm_transport::{
    ChatMessage, CompletionOk, LocalLlmTransport, OpenRouterTransport, TransportError,
};

/// Transport handle injected into Mini.
pub enum LlmTransport {
    OpenRouter(OpenRouterTransport),
    Local(LocalLlmTransport),
}

impl LlmTransport {
    /// # Errors
    ///
    /// Returns [`TransportError`] when auth or engine readiness fails.
    pub fn ensure_ready(&self) -> Result<(), TransportError> {
        match self {
            Self::OpenRouter(t) => t.ensure_ready(),
            Self::Local(t) => t.ensure_ready(),
        }
    }

    /// Complete one chat turn.
    pub async fn complete(&self, messages: &[ChatMessage]) -> Result<CompletionOk, TransportError> {
        match self {
            Self::OpenRouter(t) => t.complete(messages).await,
            Self::Local(t) => t.complete(messages).await,
        }
    }
}
