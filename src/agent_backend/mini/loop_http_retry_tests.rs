use malvin_mini::{ChatMessage, ChatRole, CompletionResponse};

use super::loop_driver::{
    complete_with_http_retries, HttpCompletionError, HttpRetryCounters, HttpRetryLimits,
    HttpRetryRequest, LlmBackend, MockScript, MockStep, RetryClass,
};
use crate::agent_backend::mini::trace::MiniTraceSink;
use crate::agent_backend::test_support::test_io;

#[tokio::test]
async fn complete_with_http_retries_reports_actual_attempt_count_for_non_retryable() {
    let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
        responses: vec![MockStep::RequestFailed {
            status: 400,
            body: "bad".into(),
        }],
        call_count: 0,
        on_response: None,
    }));
    let messages = [ChatMessage {
        role: ChatRole::User,
        content: "hi".into(),
    }];
    let err = complete_with_http_retries(HttpRetryRequest {
        llm: &llm,
        messages: &messages,
        max_api_retries: 3,
        max_transport_retries: 3,
        single_attempt: false,
        timing: None,
        trace: None,
    })
    .await
    .expect_err("400");
    assert!(matches!(
        err,
        HttpCompletionError::Exhausted(ref msg)
            if msg.contains("after 1 API attempts") && msg.contains("(limit 3)")
    ));
}

#[tokio::test]
async fn complete_with_http_retries_succeeds_on_second_mock_attempt() {
    let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
        responses: vec![
            MockStep::RateLimited,
            MockStep::Ok(CompletionResponse {
                content: "ok".into(),
                usage: None,
                reasoning: None,
            }),
        ],
        call_count: 0,
        on_response: None,
    }));
    let messages = [ChatMessage {
        role: ChatRole::User,
        content: "hi".into(),
    }];
    let resp = complete_with_http_retries(HttpRetryRequest {
        llm: &llm,
        messages: &messages,
        max_api_retries: 2,
        max_transport_retries: 3,
        single_attempt: false,
        timing: None,
        trace: None,
    })
    .await
    .expect("retry ok");
    assert_eq!(resp.content, "ok");
}

#[tokio::test]
async fn complete_with_http_retries_maps_context_overflow() {
    let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
        responses: vec![MockStep::ContextOverflow],
        call_count: 0,
        on_response: None,
    }));
    let messages = [ChatMessage {
        role: ChatRole::User,
        content: "hi".into(),
    }];
    let err = complete_with_http_retries(HttpRetryRequest {
        llm: &llm,
        messages: &messages,
        max_api_retries: 1,
        max_transport_retries: 1,
        single_attempt: true,
        timing: None,
        trace: None,
    })
    .await
    .expect_err("overflow");
    assert!(matches!(err, HttpCompletionError::ContextOverflow));
}

#[tokio::test]
async fn complete_with_http_retries_exhausts_transport_budget() {
    let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
        responses: vec![MockStep::Transport, MockStep::Transport, MockStep::Transport],
        call_count: 0,
        on_response: None,
    }));
    let messages = [ChatMessage {
        role: ChatRole::User,
        content: "hi".into(),
    }];
    let err = complete_with_http_retries(HttpRetryRequest {
        llm: &llm,
        messages: &messages,
        max_api_retries: 1,
        max_transport_retries: 3,
        single_attempt: false,
        timing: None,
        trace: None,
    })
    .await
    .expect_err("transport exhausted");
    assert!(matches!(
        err,
        HttpCompletionError::Exhausted(ref msg)
            if msg.contains("transport") && msg.contains("after 3 transport attempts")
    ));
}

#[tokio::test]
async fn complete_with_http_retries_rate_limited_uses_api_not_transport_budget() {
    let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
        responses: vec![MockStep::RateLimited, MockStep::RateLimited, MockStep::RateLimited],
        call_count: 0,
        on_response: None,
    }));
    let messages = [ChatMessage {
        role: ChatRole::User,
        content: "hi".into(),
    }];
    let err = complete_with_http_retries(HttpRetryRequest {
        llm: &llm,
        messages: &messages,
        max_api_retries: 3,
        max_transport_retries: 99,
        single_attempt: false,
        timing: None,
        trace: None,
    })
    .await
    .expect_err("api exhausted");
    assert!(matches!(
        err,
        HttpCompletionError::Exhausted(ref msg)
            if msg.contains("API") && msg.contains("after 3 API attempts")
    ));
}

#[tokio::test]
async fn complete_with_http_retries_emits_mini_http_exchange_to_trace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = MiniTraceSink::new(Some(tmp.path().to_path_buf()), test_io());
    let llm = LlmBackend::Mock(std::sync::Mutex::new(MockScript {
        responses: vec![MockStep::Ok(CompletionResponse {
            content: "ok".into(),
            usage: None,
            reasoning: None,
        })],
        call_count: 0,
        on_response: None,
    }));
    let messages = [ChatMessage {
        role: ChatRole::User,
        content: "hi".into(),
    }];
    let resp = complete_with_http_retries(HttpRetryRequest {
        llm: &llm,
        messages: &messages,
        max_api_retries: 1,
        max_transport_retries: 3,
        single_attempt: false,
        timing: None,
        trace: Some(&sink),
    })
    .await
    .expect("ok");
    assert_eq!(resp.content, "ok");
    let trace = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
    assert!(
        trace.contains("miniHttpExchange") && trace.contains("\"status\":200"),
        "trace must record HTTP exchange; got {trace:?}"
    );
}

#[test]
fn kiss_witness_http_retry_types() {
    let _ = std::mem::size_of::<RetryClass>();
    let _ = std::mem::size_of::<HttpRetryLimits>();
    let _ = std::mem::size_of::<HttpRetryCounters>();
}
