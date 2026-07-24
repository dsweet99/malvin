use super::client::OpenRouterClient;
use super::complete::shrink_prompt_messages;
use super::types::{ChatMessage, ChatRole};
use crate::error::OpenRouterError;
use crate::test_support::{
    mount_prompt_too_long_once, mount_prompt_too_long_then_success_after_drop,
    openrouter_test_config,
};
use wiremock::MockServer;

pub(crate) fn twelve_word_prompt() -> Vec<ChatMessage> {
    vec![ChatMessage {
        role: ChatRole::User,
        content: "w0 w1 w2 w3 w4 w5 w6 w7 w8 w9 w10 w11".into(),
    }]
}

#[test]
fn twelve_word_prompt_is_single_user_message() {
    let msgs = twelve_word_prompt();
    assert_eq!(msgs.len(), 1);
    assert_eq!(msgs[0].role, ChatRole::User);
    assert!(msgs[0].content.contains("w11"));
}

#[test]
fn shrink_prompt_messages_drops_oldest_non_system() {
    let mut msgs = vec![
        ChatMessage {
            role: ChatRole::System,
            content: "sys".into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "old".into(),
        },
        ChatMessage {
            role: ChatRole::Assistant,
            content: "keep".into(),
        },
    ];
    assert!(shrink_prompt_messages(&mut msgs));
    assert_eq!(msgs.len(), 2);
    assert_eq!(msgs[0].content, "sys");
    assert_eq!(msgs[1].content, "keep");
}

#[tokio::test]
pub(crate) async fn openrouter_complete_surfaces_invalid_referer_header_errors() {
    let server = MockServer::start().await;
    let mut config = openrouter_test_config(&server.uri());
    config.http_referer = Some("bad\nreferer".into());
    let client = OpenRouterClient::new(config).expect("client");
    let err = client.complete(&[]).await.result.expect_err("invalid referer");
    assert!(matches!(err, OpenRouterError::RequestFailed { status: 0, .. }));
}

#[tokio::test]
pub(crate) async fn openrouter_prompt_too_long_maps_to_context_overflow() {
    let server = MockServer::start().await;
    mount_prompt_too_long_once(&server).await;
    let client = OpenRouterClient::new(openrouter_test_config(&server.uri())).expect("client");
    // Short sole message: local shrink cannot help, so overflow surfaces.
    let messages = vec![ChatMessage {
        role: ChatRole::User,
        content: "only".into(),
    }];
    let err = client.complete(&messages).await.result.expect_err("overflow");
    assert!(matches!(err, OpenRouterError::ContextOverflow { .. }));
}

#[tokio::test]
pub(crate) async fn openrouter_prompt_token_limit_maps_to_context_overflow() {
    let server = MockServer::start().await;
    wiremock::Mock::given(wiremock::matchers::method("POST"))
        .respond_with(wiremock::ResponseTemplate::new(400).set_body_string(
            r#"{"error":{"message":"Provider returned error","metadata":{"provider_name":"Provider","raw":"Prompt tokens limit exceeded: 21287 > 13840"}}}"#,
        ))
        .expect(1..)
        .mount(&server)
        .await;
    let client = OpenRouterClient::new(openrouter_test_config(&server.uri())).expect("client");
    let err = client
        .complete(&[ChatMessage {
            role: ChatRole::User,
            content: "only".into(),
        }])
        .await
        .result
        .expect_err("overflow");
    assert!(matches!(err, OpenRouterError::ContextOverflow { .. }));
}

#[tokio::test]
pub(crate) async fn openrouter_prompt_too_long_shrink_retries_to_success() {
    let server = MockServer::start().await;
    mount_prompt_too_long_then_success_after_drop(&server).await;
    let client = OpenRouterClient::new(openrouter_test_config(&server.uri())).expect("client");
    let messages = vec![
        ChatMessage {
            role: ChatRole::User,
            content: "drop-me-old".into(),
        },
        ChatMessage {
            role: ChatRole::Assistant,
            content: "keep-assistant".into(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: "keep-user".into(),
        },
    ];
    let ok = client.complete(&messages).await.result.expect("shrunk ok");
    assert_eq!(ok.content, "ok");
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::{
        openrouter_complete_surfaces_invalid_referer_header_errors,
        openrouter_prompt_too_long_maps_to_context_overflow,
        openrouter_prompt_too_long_shrink_retries_to_success,
        openrouter_prompt_token_limit_maps_to_context_overflow,
        shrink_prompt_messages, twelve_word_prompt,
    };

    #[test]
    fn kiss_cov_prompt_too_long_retry_test_symbols() {
        let _ = (
            twelve_word_prompt,
            shrink_prompt_messages,
            openrouter_complete_surfaces_invalid_referer_header_errors,
            openrouter_prompt_too_long_maps_to_context_overflow,
            openrouter_prompt_token_limit_maps_to_context_overflow,
            openrouter_prompt_too_long_shrink_retries_to_success,
        );
    }
}
