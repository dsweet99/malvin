//! External kiss witnesses for malvin-mini openrouter modules.

use super::client::OpenRouterClient;
use super::openrouter_tests;
use super::prompt_too_long_retry_tests;

#[test]
fn kiss_witness_openrouter_test_fns() {
    let _ = OpenRouterClient::complete;
    let _ = (
        openrouter_tests::openrouter_serializes_model_messages_and_headers,
        openrouter_tests::openrouter_error_maps_401_unauthorized,
        openrouter_tests::openrouter_error_maps_429_rate_limit,
        openrouter_tests::openrouter_error_maps_500_server_error,
        openrouter_tests::openrouter_error_maps_billing_failure,
        openrouter_tests::openrouter_mock_http_complete_returns_usage,
        openrouter_tests::openrouter_mock_http_complete_returns_usage_cost,
        openrouter_tests::openrouter_error_on_non_200_request_failed,
        openrouter_tests::openrouter_error_on_missing_content,
        prompt_too_long_retry_tests::twelve_word_prompt,
        prompt_too_long_retry_tests::openrouter_complete_surfaces_invalid_referer_header_errors,
        prompt_too_long_retry_tests::openrouter_prompt_too_long_maps_to_context_overflow,
        prompt_too_long_retry_tests::openrouter_prompt_too_long_does_not_retry_in_transport,
    );
}

#[test]
fn kiss_witness_openrouter_serde_types() {
    use super::serde_types::{
        ChatChoice, ChatChoiceMessage, ChatCompletionRequest, ChatCompletionResponse,
    };
    use super::types::{ChatMessage, ChatRole, ResponseUsage};

    let msgs = [ChatMessage {
        role: ChatRole::User,
        content: "hi".into(),
    }];
    let req = ChatCompletionRequest {
        model: "m",
        messages: &msgs,
    };
    let ChatCompletionRequest { model, messages } = req;
    assert_eq!(model, "m");
    assert_eq!(messages.len(), 1);

    let resp = ChatCompletionResponse {
        choices: vec![],
        usage: None,
    };
    let ChatCompletionResponse { choices, usage } = resp;
    assert!(choices.is_empty());
    assert!(usage.is_none());

    let choice = ChatChoice { message: None };
    let ChatChoice { message } = choice;
    assert!(message.is_none());

    let msg: ChatChoiceMessage =
        serde_json::from_str(r#"{"content":"c","reasoning":null}"#).expect("msg");
    assert_eq!(msg.text_content().as_deref(), Some("c"));

    let usage = ResponseUsage {
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: Some(1),
        cost: None,
    };
    let ResponseUsage { total_tokens, .. } = usage;
    assert_eq!(total_tokens, Some(1));

    let _ = stringify!(deserialize_message_content);
    let _ = stringify!(deserialize_message_content_accepts_text_and_parts);
}
