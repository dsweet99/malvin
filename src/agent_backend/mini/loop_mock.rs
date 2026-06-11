//! Mock LLM backend for mini bash-loop tests.

use malvin_mini::{ChatMessage, CompletionResponse, OpenRouterError};

pub enum MockStep {
    Ok(CompletionResponse),
    RateLimited,
}

pub struct MockScript {
    pub responses: Vec<MockStep>,
    pub call_count: usize,
    #[cfg(test)]
    pub on_response: Option<Box<dyn FnMut(usize) + Send>>,
}

pub enum LlmBackend {
    Http(malvin_mini::OpenRouterClient),
    Mock(std::sync::Mutex<MockScript>),
}

impl LlmBackend {
    pub async fn complete(&self, messages: &[ChatMessage]) -> Result<CompletionResponse, OpenRouterError> {
        match self {
            Self::Http(client) => client.complete(messages).await,
            Self::Mock(script) => {
                let mut g = script.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                let idx = g.call_count;
                g.call_count += 1;
                #[cfg(test)]
                if let Some(ref mut hook) = g.on_response {
                    hook(idx);
                }
                match g.responses.get(idx) {
                    Some(MockStep::Ok(r)) => Ok(r.clone()),
                    Some(MockStep::RateLimited) => {
                        Err(OpenRouterError::RateLimited { body: "slow".into() })
                    }
                    None => Err(OpenRouterError::RequestFailed {
                        status: 0,
                        body: "mock script exhausted".into(),
                    }),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use malvin_mini::{ChatMessage, ChatRole, CompletionResponse};

    use super::{LlmBackend, MockScript, MockStep};

    #[tokio::test]
    async fn mock_llm_backend_returns_scripted_responses() {
        let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![
                MockStep::Ok(CompletionResponse {
                    content: "a".into(),
                    usage: None,
                }),
                MockStep::RateLimited,
            ],
            call_count: 0,
            on_response: None,
        }));
        let messages = [ChatMessage {
            role: ChatRole::User,
            content: "hi".into(),
        }];
        let first = llm.complete(&messages).await.expect("first");
        assert_eq!(first.content, "a");
        let second = llm.complete(&messages).await.expect_err("rate limited");
        assert!(second.is_retryable());
    }
}
