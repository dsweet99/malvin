//! Mock LLM backend for mini bash-loop tests.

use crate::openrouter_transport::{
    ChatMessage, CompletionResponse, HttpExchangeMeta, TransportError,
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
    pub result: Result<CompletionResponse, TransportError>,
    pub http: HttpExchangeMeta,
}

pub enum LlmBackend {
    Http(crate::openrouter_transport::OpenRouterClient),
    Local(crate::local_llm::LocalCompletionEngine),
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


async fn complete_http_with_protocol(
    client: &crate::openrouter_transport::OpenRouterClient,
    messages: &[ChatMessage],
) -> crate::openrouter_transport::CompletionWithMeta {
    crate::mini_agent::protocol::complete_with_protocol_shape(messages, |msgs| {
        async move { client.complete_http(&msgs).await }
    })
    .await
}

async fn complete_local_with_protocol(
    engine: &crate::local_llm::LocalCompletionEngine,
    messages: &[ChatMessage],
) -> crate::openrouter_transport::CompletionWithMeta {
    crate::mini_agent::protocol::complete_with_protocol_shape(messages, |msgs| {
        async move {
            let (result, http) = engine.complete(&msgs).await;
            crate::openrouter_transport::CompletionWithMeta { result, http }
        }
    })
    .await
}

impl LlmBackend {
    pub async fn complete(&self, messages: &[ChatMessage]) -> LlmCompletionOutcome {
        match self {
            Self::Http(client) => {
                let meta = complete_http_with_protocol(client, messages).await;
                LlmCompletionOutcome {
                    result: meta.result,
                    http: meta.http,
                }
            }
            Self::Local(engine) => {
                let meta = complete_local_with_protocol(engine, messages).await;
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
                        result: Err(TransportError::RequestFailed {
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
    use super::{LlmCompletionOutcome, MockScript, MockStep};
    use crate::openrouter_transport::{CompletionResponse, HttpExchangeMeta};

    #[tokio::test]
    async fn mock_llm_backend_returns_scripted_responses() {
        use super::LlmBackend;
        use crate::openrouter_transport::{ChatMessage, ChatRole};
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
        let messages = [ChatMessage { role: ChatRole::User, content: "hi".into() }];
        assert_eq!(llm.complete(&messages).await.result.expect("first").content, "a");
        assert!(llm.complete(&messages).await.result.expect_err("rl").is_transport_retryable());
    }

    #[test]
    fn kiss_witness_mock_units() {
        let _ = (
            stringify!(MockStep),
            stringify!(MockScript),
            stringify!(LlmCompletionOutcome),
            stringify!(complete_http_with_protocol),
            stringify!(complete_local_with_protocol),
            super::complete_http_with_protocol,
            super::complete_local_with_protocol,
            super::mock_step_outcome,
            stringify!(on_response),
        );
        let _ = LlmCompletionOutcome {
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
        let _ = MockScript {
            responses: vec![MockStep::RateLimited],
            call_count: 0,
            on_response: None,
        };
    }
}
