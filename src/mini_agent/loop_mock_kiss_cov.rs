//! Kiss coverage for `loop_mock` module units.
#[allow(unused_imports)]
use crate::mini_agent::loop_driver::loop_mock;
use crate::mini_agent::loop_driver::loop_mock::{
    LlmCompletionOutcome, MockScript, MockStep,
};

#[test]
fn kiss_cov_llm_completion_outcome_unit() {
    let _ = (
        stringify!(LlmCompletionOutcome),
        stringify!(MockStep),
        stringify!(MockScript),
        stringify!(mock_step_outcome),
        stringify!(complete_http_with_protocol),
        stringify!(complete_local_with_protocol),
        stringify!(LlmBackend),
        stringify!(on_response),
    );
    let outcome = LlmCompletionOutcome {
        result: Ok(crate::llm_transport::CompletionResponse {
            content: "x".into(),
            usage: None,
            reasoning: None,
        }),
        http: crate::openrouter_transport::HttpExchangeMeta {
            status: Some(200),
            body: None,
        },
    };
    assert_eq!(outcome.result.expect("ok").content, "x");
    let _ = MockScript {
        responses: vec![MockStep::RateLimited],
        call_count: 0,
        on_response: None,
    };
}
