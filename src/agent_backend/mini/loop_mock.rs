//! Mock LLM backend for mini bash-loop tests.

use malvin_mini::{
    ChatMessage, CompletionResponse, HttpExchangeMeta, OpenRouterError,
};

use super::loop_mock_outcomes::{
    mock_billing_failure_pair, mock_context_overflow_pair, mock_json_transport_pair, mock_ok_pair,
    mock_provider_fatal_pair, mock_provider_transport_pair, mock_rate_limited_pair,
    mock_request_failed_pair,
};

pub enum MockStep {
    Ok(CompletionResponse),
    RateLimited,
    ContextOverflow,
    RequestFailed { status: u16, body: String },
    BillingFailure { status: u16, body: String },
    Transport,
    Json,
    ProviderTransport,
    ProviderFatal,
}

#[cfg(test)]
pub type MockResponseHook = Box<dyn FnMut(usize, &[ChatMessage]) + Send>;

pub struct MockScript {
    pub responses: Vec<MockStep>,
    pub call_count: usize,
    #[cfg(test)]
    pub on_response: Option<MockResponseHook>,
}

pub struct LlmCompletionOutcome {
    pub result: Result<CompletionResponse, OpenRouterError>,
    pub http: HttpExchangeMeta,
}

pub enum LlmBackend {
    Http(malvin_mini::OpenRouterClient),
    Mock(std::sync::Mutex<MockScript>),
}

fn mock_step_outcome(step: &MockStep, messages: &[ChatMessage]) -> LlmCompletionOutcome {
    let (result, http) = match step {
        MockStep::Ok(r) => mock_ok_pair(r),
        MockStep::RateLimited => mock_rate_limited_pair(),
        MockStep::ContextOverflow => mock_context_overflow_pair(messages.len()),
        MockStep::RequestFailed { status, body } => mock_request_failed_pair(*status, body),
        MockStep::BillingFailure { status, body } => mock_billing_failure_pair(*status, body),
        MockStep::Transport | MockStep::Json => mock_json_transport_pair(),
        MockStep::ProviderTransport => mock_provider_transport_pair(),
        MockStep::ProviderFatal => mock_provider_fatal_pair(),
    };
    LlmCompletionOutcome { result, http }
}

impl LlmBackend {
    pub async fn complete(&self, messages: &[ChatMessage]) -> LlmCompletionOutcome {
        match self {
            Self::Http(client) => {
                let meta = client.complete(messages).await;
                LlmCompletionOutcome {
                    result: meta.result,
                    http: meta.http,
                }
            }
            Self::Mock(script) => {
                let mut g = script.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                let idx = g.call_count;
                g.call_count += 1;
                #[cfg(test)]
                if let Some(ref mut hook) = g.on_response {
                    hook(idx, messages);
                }
                g.responses.get(idx).map_or_else(
                    || LlmCompletionOutcome {
                        result: Err(OpenRouterError::RequestFailed {
                            status: 0,
                            body: "mock script exhausted".into(),
                        }),
                        http: HttpExchangeMeta {
                            status: None,
                            body: None,
                        },
                    },
                    |step| mock_step_outcome(step, messages),
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use malvin_mini::{ChatMessage, ChatRole, CompletionResponse};

    use super::{LlmBackend, LlmCompletionOutcome, MockScript, MockStep};
    use malvin_mini::HttpExchangeMeta;

    #[tokio::test]
    async fn mock_llm_backend_returns_scripted_responses() {
        let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![
                MockStep::Ok(CompletionResponse {
                    content: "a".into(),
                    usage: None,
                    reasoning: None,
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
        let first = llm.complete(&messages).await.result.expect("first");
        assert_eq!(first.content, "a");
        let second = llm.complete(&messages).await.result.expect_err("rate limited");
        assert!(second.is_transport_retryable());
    }

    #[test]
    fn kiss_witness_mock_step_outcome() {
        let _ = super::mock_step_outcome;
    }

    #[test]
    fn kiss_witness_llm_completion_outcome_type() {
        let outcome = LlmCompletionOutcome {
            result: Ok(CompletionResponse {
                content: "x".into(),
                usage: None,
                reasoning: None,
            }),
            http: HttpExchangeMeta {
                status: Some(200),
                body: None,
            },
        };
        let LlmCompletionOutcome { result, http } = outcome;
        assert_eq!(result.expect("ok").content, "x");
        assert_eq!(http.status, Some(200));
    }
}
