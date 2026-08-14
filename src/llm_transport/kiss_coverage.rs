
#[test]
fn kiss_cov_llm_transport_types_and_errors() {
    use crate::llm_transport::{
        body_indicates_prompt_too_long, is_prompt_too_long_error, ChatMessage, ChatRole,
        CompletionMeta, CompletionOk, CompletionResponse, HttpExchangeMeta, ResponseUsage,
        TransportError,
    };

    let msg = ChatMessage {
        role: ChatRole::User,
        content: "hi".into(),
    };
    let ok = CompletionOk {
        content: "x".into(),
        meta: CompletionMeta::default(),
    };
    assert_eq!(ok.clone().into_response().content, "x");
    let from = CompletionOk::from(CompletionResponse {
        content: "y".into(),
        usage: Some(ResponseUsage {
            prompt_tokens: Some(1),
            completion_tokens: None,
            total_tokens: None,
            cost: None,
        }),
        reasoning: None,
    });
    assert_eq!(from.content, "y");
    let http = HttpExchangeMeta {
        status: Some(200),
        body: Some("ok".into()),
    };
    let _ = (
        msg,
        http,
        TransportError::MissingContent,
        body_indicates_prompt_too_long,
        is_prompt_too_long_error,
    );
}
