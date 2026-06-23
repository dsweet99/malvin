//! External kiss witnesses for malvin-mini openrouter modules.

use super::client::OpenRouterClient;

#[test]
fn kiss_witness_openrouter_test_fns() {
    let _ = OpenRouterClient::complete;
    let _ = super::openrouter_tests::openrouter_serializes_model_messages_and_headers;
    let _ = super::openrouter_tests::openrouter_error_maps_401_unauthorized;
    let _ = super::openrouter_tests::openrouter_error_maps_429_rate_limit;
    let _ = super::openrouter_tests::openrouter_error_maps_500_server_error;
    let _ = super::openrouter_tests::openrouter_error_maps_billing_failure;
    let _ = super::openrouter_tests::openrouter_mock_http_complete_returns_usage;
    let _ = super::openrouter_tests::openrouter_mock_http_complete_returns_usage_cost;
    let _ = super::openrouter_tests::openrouter_error_on_non_200_request_failed;
    let _ = super::openrouter_tests::openrouter_error_on_missing_content;
    let _ = super::prompt_too_long_retry_tests::twelve_word_prompt;
    let _ = super::prompt_too_long_retry_tests::openrouter_complete_surfaces_invalid_referer_header_errors;
    let _ = super::prompt_too_long_retry_tests::openrouter_prompt_too_long_stops_when_shrink_makes_no_change;
    let _ = super::prompt_too_long_retry_tests::openrouter_retries_after_prompt_too_long_by_shrinking_middle_odd_words;
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

    let msg = ChatChoiceMessage {
        content: Some("c".into()),
    };
    let ChatChoiceMessage { content } = msg;
    assert_eq!(content.as_deref(), Some("c"));

    let usage = ResponseUsage {
        prompt_tokens: None,
        completion_tokens: None,
        total_tokens: Some(1),
        cost: None,
    };
    let ResponseUsage { total_tokens, .. } = usage;
    assert_eq!(total_tokens, Some(1));
}
