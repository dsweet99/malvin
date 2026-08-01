//! Contract: `OpenRouter` types remain usable after `malvin-mini` was folded into `malvin`.

use malvin::malvin_mini::{ChatMessage, ChatRole, CompletionResponse, ResponseUsage};

#[test]
fn openrouter_types_roundtrip() {
    let role = ChatRole::User;
    let json = serde_json::to_string(&role).expect("role");
    assert_eq!(json, "\"user\"");

    let msg = ChatMessage {
        role: ChatRole::User,
        content: "hi".into(),
    };
    let ChatMessage { role, content } = msg;
    assert_eq!(role, ChatRole::User);
    assert_eq!(content, "hi");

    let usage = ResponseUsage {
        prompt_tokens: Some(1),
        completion_tokens: Some(2),
        total_tokens: Some(3),
        cost: Some(0.1),
    };
    let usage_json = serde_json::to_string(&usage).expect("usage");
    assert!(usage_json.contains("prompt_tokens"));

    let resp = CompletionResponse {
        content: "done".into(),
        usage: Some(usage),
        reasoning: None,
    };
    let CompletionResponse {
        content,
        usage,
        reasoning,
    } = resp;
    assert_eq!(content, "done");
    assert!(usage.is_some());
    assert!(reasoning.is_none());
}
