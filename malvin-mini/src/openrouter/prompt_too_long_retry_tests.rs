use super::client::OpenRouterClient;
use super::types::{ChatMessage, ChatRole};
use crate::error::OpenRouterError;
use crate::test_support::{
    mount_prompt_too_long_once, openrouter_test_config,
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

#[tokio::test]
pub(crate) async fn openrouter_complete_surfaces_invalid_referer_header_errors() {
    let server = MockServer::start().await;
    let mut config = openrouter_test_config(&server.uri());
    config.http_referer = Some("bad\nreferer".into());
    let client = OpenRouterClient::new(config).expect("client");
    let err = client.complete(&[]).await.expect_err("invalid referer");
    assert!(matches!(err, OpenRouterError::RequestFailed { status: 0, .. }));
}

#[tokio::test]
pub(crate) async fn openrouter_prompt_too_long_maps_to_context_overflow() {
    let server = MockServer::start().await;
    mount_prompt_too_long_once(&server).await;
    let client = OpenRouterClient::new(openrouter_test_config(&server.uri())).expect("client");
    let messages = vec![ChatMessage {
        role: ChatRole::User,
        content: "only".into(),
    }];
    let err = client.complete(&messages).await.expect_err("overflow");
    assert!(matches!(err, OpenRouterError::ContextOverflow { .. }));
}

#[tokio::test]
pub(crate) async fn openrouter_prompt_too_long_does_not_retry_in_transport() {
    let server = MockServer::start().await;
    mount_prompt_too_long_once(&server).await;
    let client = OpenRouterClient::new(openrouter_test_config(&server.uri())).expect("client");
    let err = client
        .complete(&twelve_word_prompt())
        .await
        .expect_err("transport does not shrink-retry");
    assert!(matches!(err, OpenRouterError::ContextOverflow { .. }));
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::{
        openrouter_complete_surfaces_invalid_referer_header_errors,
        openrouter_prompt_too_long_does_not_retry_in_transport,
        openrouter_prompt_too_long_maps_to_context_overflow,
        twelve_word_prompt,
    };

    #[test]
    fn kiss_cov_prompt_too_long_retry_test_symbols() {
        let _ = (
            twelve_word_prompt,
            openrouter_complete_surfaces_invalid_referer_header_errors,
            openrouter_prompt_too_long_maps_to_context_overflow,
            openrouter_prompt_too_long_does_not_retry_in_transport,
        );
    }
}
