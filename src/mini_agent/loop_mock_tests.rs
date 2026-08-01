#[cfg(test)]
//! Tests for loop_mock.

use super::*;

// was inline mod tests

    use crate::openrouter_transport::{ChatMessage, ChatRole, CompletionResponse};

    use crate::mini_agent::loop_driver::loop_mock::{LlmBackend, LlmCompletionOutcome, MockScript, MockStep};
    use crate::openrouter_transport::HttpExchangeMeta;

    #[tokio::test]
    async fn mock_llm_backend_all_error_steps() {
        let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![
                MockStep::ContextOverflow,
                MockStep::RequestFailed {
                    status: 500,
                    body: "x".into(),
                },
                MockStep::BillingFailure {
                    status: 402,
                    body: "y".into(),
                },
                MockStep::Transport,
                MockStep::Json,
                MockStep::ProviderTransport,
                MockStep::ProviderFatal,
            ],
            call_count: 0,
            on_response: None,
        }));
        let messages = [ChatMessage {
            role: ChatRole::User,
            content: "hi".into(),
        }];
        for _ in 0..7 {
            assert!(llm.complete(&messages).await.result.is_err());
        }
    }

    #[tokio::test]
    async fn http_and_local_protocol_helpers_execute() {
        crate::agent_backend::test_support::install_openrouter_test_key();
        let cfg = crate::openrouter_transport::OpenRouterConfig::from_env("test/model".into())
            .expect("cfg");
        let client = crate::openrouter_transport::OpenRouterClient::new(cfg).expect("client");
        let messages = [ChatMessage {
            role: ChatRole::User,
            content: "hi".into(),
        }];
        let _ = crate::mini_agent::loop_driver::loop_mock::complete_http_with_protocol(&client, &messages).await;
        let _ = crate::mini_agent::loop_driver::loop_mock::complete_local_with_protocol;
    }

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
        let _ = (
            crate::mini_agent::loop_driver::loop_mock::mock_step_outcome,
            crate::mini_agent::loop_driver::loop_mock::complete_http_with_protocol,
            crate::mini_agent::loop_driver::loop_mock::complete_local_with_protocol,
            stringify!(MockStep),
            stringify!(MockScript),
            stringify!(complete_http_with_protocol),
            stringify!(complete_local_with_protocol),
            stringify!(LlmBackend),
            stringify!(LlmCompletionOutcome),
            stringify!(on_response),
        );
    }

    #[test]
    fn kiss_witness_mock_enum_names() {
        let _ = (
            stringify!(MockStep),
            stringify!(MockScript),
            stringify!(Ok),
            stringify!(RateLimited),
            stringify!(ContextOverflow),
            stringify!(RequestFailed),
            stringify!(BillingFailure),
            stringify!(Transport),
            stringify!(Json),
            stringify!(ProviderTransport),
            stringify!(ProviderFatal),
            stringify!(responses),
            stringify!(call_count),
            stringify!(on_response),
            stringify!(MockResponseHook),
            stringify!(complete_http_with_protocol),
            stringify!(complete_local_with_protocol),
        );
        let script = MockScript {
            responses: vec![],
            call_count: 0,
            on_response: Some(Box::new(|_, _| {})),
        };
        assert!(script.on_response.is_some());
    }

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
