use super::complete::{completion_with_meta, finalize_complete_outcome, transport_meta};
use super::client::OpenRouterClient;
use crate::malvin_mini::types::CompletionResponse;

#[test]
fn finalize_complete_outcome_passthrough() {
    let pass = completion_with_meta(
        Ok(CompletionResponse {
            content: "ok".into(),
            usage: None,
            reasoning: None,
        }),
        transport_meta(Some(200), None),
    );
    let out = finalize_complete_outcome(pass, &[]);
    assert_eq!(out.result.as_ref().expect("ok").content, "ok");
    let _ = OpenRouterClient::complete_with_max_tokens;
    let _ = stringify!(post_chat_completion);
    let _ = stringify!(body_has_reasoning);
}
