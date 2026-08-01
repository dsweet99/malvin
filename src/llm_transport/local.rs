//! Local LLM transport.

use crate::llm_transport::{ChatMessage, CompletionMeta, CompletionOk, TransportError};
use crate::local_llm::LocalCompletionEngine;

/// In-process local completer transport.
pub struct LocalLlmTransport {
    engine: LocalCompletionEngine,
}

impl LocalLlmTransport {
    #[must_use]
    pub const fn new(engine: LocalCompletionEngine) -> Self {
        Self { engine }
    }

    #[must_use]
    pub const fn engine(&self) -> &LocalCompletionEngine {
        &self.engine
    }

    #[must_use]
    pub fn into_engine(self) -> LocalCompletionEngine {
        self.engine
    }

    /// # Errors
    ///
    /// Local engine is always ready after construction.
    pub const fn ensure_ready(&self) -> Result<(), TransportError> {
        Ok(())
    }

    /// Complete one turn via the in-process engine.
    pub async fn complete(&self, messages: &[ChatMessage]) -> Result<CompletionOk, TransportError> {
        let (result, http) = self.engine.complete(messages).await;
        result.map(|r| CompletionOk {
            content: r.content,
            meta: CompletionMeta {
                status: http.status,
                body: http.body,
                usage: r.usage,
                reasoning: r.reasoning,
            },
        })
    }
}
