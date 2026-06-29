//! External kiss witnesses for malvin-mini openrouter modules.

use super::client::OpenRouterClient;
use super::openrouter_tests;
use super::prompt_too_long_retry_tests;

#[test]
fn kiss_witness_openrouter_test_fns() {
    let _ = OpenRouterClient::complete;
    let _ = OpenRouterClient::list_models;
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
        openrouter_tests::openrouter_complete_transport_error_on_unreachable_host,
        super::fetch_completion_tests::fetch_completion_body_maps_http_200_non_retryable_provider_error,
        super::fetch_completion_tests::fetch_completion_body_maps_http_200_nvidia_resource_exhausted,
        super::fetch_completion_tests::fetch_completion_body_surfaces_transport_errors,
        super::fetch_completion_tests::fetch_completion_body_surfaces_header_validation_errors,
        super::fetch_completion_tests::fetch_completion_body_reads_success_body,
        super::list_models_tests::list_models_parses_success_response,
        super::list_models_tests::list_models_maps_401_to_unauthorized,
        super::list_models_tests::list_models_maps_500_to_server_error,
        super::list_models_tests::list_models_works_without_api_key,
        prompt_too_long_retry_tests::twelve_word_prompt,
        prompt_too_long_retry_tests::openrouter_complete_surfaces_invalid_referer_header_errors,
        prompt_too_long_retry_tests::openrouter_prompt_too_long_maps_to_context_overflow,
        prompt_too_long_retry_tests::openrouter_prompt_too_long_does_not_retry_in_transport,
    );
}

#[test]
fn kiss_witness_openrouter_serde_types() {
    kiss_witness_openrouter_request_response_types();
    kiss_witness_openrouter_http_exchange_types();
}

fn kiss_witness_openrouter_request_response_types() {
    use super::list_models::ModelListing;
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

    let listing = ModelListing {
        id: "id".into(),
        name: "name".into(),
    };
    assert_eq!(listing.id, "id");
}

fn kiss_witness_openrouter_http_exchange_types() {
    use crate::error::OpenRouterError;

    let http = super::http_exchange::HttpExchangeMeta {
        status: Some(200),
        body: Some("body".into()),
    };
    let super::http_exchange::HttpExchangeMeta { status, body } = http;
    assert_eq!(status, Some(200));
    assert_eq!(body.as_deref(), Some("body"));

    let with_meta = super::http_exchange::CompletionWithMeta {
        result: Ok(super::types::CompletionResponse {
            content: "ok".into(),
            usage: None,
            reasoning: None,
        }),
        http: super::http_exchange::HttpExchangeMeta {
            status: Some(200),
            body: None,
        },
    };
    let super::http_exchange::CompletionWithMeta { result, http } = with_meta;
    assert_eq!(result.as_ref().expect("ok").content, "ok");
    assert_eq!(http.status, Some(200));
    let err_meta = super::http_exchange::CompletionWithMeta {
        result: Err(OpenRouterError::MissingContent),
        http: super::http_exchange::HttpExchangeMeta {
            status: Some(500),
            body: Some("err".into()),
        },
    };
    assert!(err_meta.result.is_err());
    assert_eq!(err_meta.http.body.as_deref(), Some("err"));
    let _ = stringify!(completion_with_meta_exposes_result_and_http);
    let _ = super::complete::completion_with_meta;
    let _ = super::complete::transport_meta;
    let _ = super::complete::transport_failure_meta;
    let _ = stringify!(outcome_from_http_body);
    let _ = stringify!(provider_fatal_from_body);
    let _ = super::provider_error::provider_fatal_from_body;
    let _ = stringify!(provider_transport_from_body);
    let _ = super::provider_error::provider_transport_from_body;
    let _ = stringify!(ModelListing);
    let _ = stringify!(list_models_url);
    let _ = stringify!(completion_with_meta_and_transport_meta_helpers);
    let _ = stringify!(kiss_witness_completion_post_url);
    let _ = stringify!(kiss_witness_transport_failure_meta);
}
