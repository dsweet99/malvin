use malvin_mini::{ChatMessage, ChatRole, CompletionResponse};

use super::loop_driver::{
    complete_with_http_retries, HttpCompletionError, HttpRetryCounters, HttpRetryLimits,
    HttpRetryRequest, LlmBackend, MockStep,
};
use crate::agent_backend::mini::trace::MiniTraceSink;
use crate::agent_backend::test_support::{mock_llm, test_io};

const TRANSPORT_LIMIT: u32 = 3;

fn hi_messages() -> [ChatMessage; 1] {
    [ChatMessage {
        role: ChatRole::User,
        content: "hi".into(),
    }]
}

async fn run_http_retries(llm: &LlmBackend, limit: u32, single_attempt: bool) -> Result<CompletionResponse, HttpCompletionError> {
    complete_with_http_retries(HttpRetryRequest {
        llm,
        messages: &hi_messages(),
        max_transport_retries: limit,
        single_attempt,
        timing: None,
        trace: None,
    })
    .await
}

fn assert_transport_exhausted(err: HttpCompletionError) {
    assert!(matches!(
        err,
        HttpCompletionError::Exhausted(ref msg)
            if msg.contains("transport") && msg.contains("after 3 transport attempts")
    ));
}

#[tokio::test]
async fn complete_with_http_retries_non_billing_errors_exhaust_transport_budget() {
    let cases = [
        vec![MockStep::Transport, MockStep::Transport, MockStep::Transport],
        vec![MockStep::RateLimited, MockStep::RateLimited, MockStep::RateLimited],
        vec![
            MockStep::RequestFailed {
                status: 400,
                body: "bad".into(),
            },
            MockStep::RequestFailed {
                status: 400,
                body: "bad".into(),
            },
            MockStep::RequestFailed {
                status: 400,
                body: "bad".into(),
            },
        ],
    ];
    for responses in cases {
        let llm = mock_llm(responses);
        let err = run_http_retries(&llm, TRANSPORT_LIMIT, false)
            .await
            .expect_err("transport budget exhausted");
        assert_transport_exhausted(err);
    }
}

#[tokio::test]
async fn complete_with_http_retries_succeeds_on_second_mock_attempt() {
    let llm = mock_llm(vec![
        MockStep::RateLimited,
        MockStep::Ok(CompletionResponse {
            content: "ok".into(),
            usage: None,
            reasoning: None,
        }),
    ]);
    let resp = run_http_retries(&llm, TRANSPORT_LIMIT, false)
        .await
        .expect("retry ok");
    assert_eq!(resp.content, "ok");
}

#[tokio::test]
async fn complete_with_http_retries_maps_context_overflow() {
    let llm = mock_llm(vec![MockStep::ContextOverflow]);
    let err = run_http_retries(&llm, 1, true)
        .await
        .expect_err("overflow");
    assert!(matches!(err, HttpCompletionError::ContextOverflow));
}

#[tokio::test]
async fn complete_with_http_retries_retries_nvidia_resource_exhausted() {
    let llm = mock_llm(vec![
        MockStep::ProviderTransport,
        MockStep::Ok(CompletionResponse {
            content: "ok".into(),
            usage: None,
            reasoning: None,
        }),
    ]);
    let resp = run_http_retries(&llm, TRANSPORT_LIMIT, false)
        .await
        .expect("provider transport retried");
    assert_eq!(resp.content, "ok");
}

#[tokio::test]
async fn complete_with_http_retries_billing_failure_fails_on_first_attempt() {
    let llm = mock_llm(vec![
        MockStep::BillingFailure {
            status: 402,
            body: "no credits".into(),
        },
        MockStep::Ok(CompletionResponse {
            content: "should not reach".into(),
            usage: None,
            reasoning: None,
        }),
    ]);
    let err = run_http_retries(&llm, TRANSPORT_LIMIT, false)
        .await
        .expect_err("billing");
    assert!(matches!(err, HttpCompletionError::Exhausted(_)));
    let call_count = {
        let LlmBackend::Mock(m) = &llm else {
            panic!("mock llm");
        };
        m.lock().expect("lock").call_count
    };
    assert_eq!(call_count, 1, "billing failure must not retry");
}

#[tokio::test]
async fn complete_with_http_retries_emits_mini_http_exchange_to_trace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let sink = MiniTraceSink::new(Some(tmp.path().to_path_buf()), test_io());
    let llm = mock_llm(vec![MockStep::Ok(CompletionResponse {
        content: "ok".into(),
        usage: None,
        reasoning: None,
    })]);
    let resp = complete_with_http_retries(HttpRetryRequest {
        llm: &llm,
        messages: &hi_messages(),
        max_transport_retries: TRANSPORT_LIMIT,
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
    let _ = std::mem::size_of::<HttpRetryLimits>();
    let _ = std::mem::size_of::<HttpRetryCounters>();
}
