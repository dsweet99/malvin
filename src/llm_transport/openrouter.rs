//! `OpenRouter` HTTP transport.

use crate::llm_transport::{ChatMessage, CompletionMeta, CompletionOk, TransportError};
use crate::openrouter_transport::{HttpExchangeMeta, OpenRouterClient, OpenRouterConfig};

/// `OpenRouter` (or compatible) HTTP chat-completions transport.
pub struct OpenRouterTransport {
    client: OpenRouterClient,
}

impl OpenRouterTransport {
    /// # Errors
    ///
    /// Returns an error when the underlying HTTP client cannot be constructed.
    pub fn new(config: OpenRouterConfig) -> Result<Self, String> {
        let client = OpenRouterClient::new(config).map_err(|e| e.to_string())?;
        Ok(Self { client })
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) const fn client(&self) -> &OpenRouterClient {
        &self.client
    }

    #[must_use]
    #[cfg(test)]
    pub(crate) fn into_client(self) -> OpenRouterClient {
        self.client
    }

    /// # Errors
    ///
    /// Returns [`TransportError`] when the API key is missing.
    pub fn ensure_ready(&self) -> Result<(), TransportError> {
        if std::env::var("OPENROUTER_API_KEY").is_ok() {
            Ok(())
        } else {
            Err(TransportError::Unauthorized {
                body: "OPENROUTER_API_KEY is not set".into(),
            })
        }
    }

    /// HTTP completion (provider retries inside the client).
    pub async fn complete(&self, messages: &[ChatMessage]) -> Result<CompletionOk, TransportError> {
        let meta = self.client.complete_http(messages).await;
        map_completion(meta.result, meta.http)
    }
}

pub fn map_completion(
    result: Result<crate::llm_transport::CompletionResponse, TransportError>,
    http: HttpExchangeMeta,
) -> Result<CompletionOk, TransportError> {
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

#[cfg(test)]
mod tests {
    use super::map_completion;
    use crate::llm_transport::{CompletionResponse, TransportError};
    use crate::openrouter_transport::HttpExchangeMeta;

    #[test]
    fn map_completion_ok_preserves_content() {
        let ok = map_completion(
            Ok(CompletionResponse {
                content: "hi".into(),
                usage: None,
                reasoning: None,
            }),
            HttpExchangeMeta {
                status: Some(200),
                body: None,
            },
        )
        .expect("ok");
        assert_eq!(ok.content, "hi");
        assert_eq!(ok.meta.status, Some(200));
    }

    #[test]
    fn map_completion_err_passes_through() {
        let err = map_completion(
            Err(TransportError::MissingContent),
            HttpExchangeMeta {
                status: None,
                body: None,
            },
        );
        assert!(matches!(err, Err(TransportError::MissingContent)));
    }
}
