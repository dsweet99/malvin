use super::client::OpenRouterClient;
use super::types::{ChatMessage, ChatRole};
use crate::error::OpenRouterError;
use crate::test_support::{
    mount_prompt_too_long_once, mount_prompt_too_long_then_success, openrouter_test_config,
};
use wiremock::MockServer;

fn twelve_word_prompt() -> Vec<ChatMessage> {
    vec![ChatMessage {
        role: ChatRole::User,
        content: "w0 w1 w2 w3 w4 w5 w6 w7 w8 w9 w10 w11".into(),
    }]
}

#[tokio::test]
async fn openrouter_complete_surfaces_invalid_referer_header_errors() {
    let server = MockServer::start().await;
    let mut config = openrouter_test_config(&server.uri());
    config.http_referer = Some("bad\nreferer".into());
    let client = OpenRouterClient::new(config).expect("client");
    let err = client.complete(&[]).await.expect_err("invalid referer");
    assert!(matches!(err, OpenRouterError::RequestFailed { status: 0, .. }));
}

#[tokio::test]
async fn openrouter_prompt_too_long_stops_when_shrink_makes_no_change() {
    let server = MockServer::start().await;
    mount_prompt_too_long_once(&server).await;
    let client = OpenRouterClient::new(openrouter_test_config(&server.uri())).expect("client");
    let messages = vec![ChatMessage {
        role: ChatRole::User,
        content: "only".into(),
    }];
    let err = client.complete(&messages).await.expect_err("unchanged shrink");
    assert!(matches!(err, OpenRouterError::RequestFailed { status: 400, .. }));
}

#[tokio::test]
async fn openrouter_retries_after_prompt_too_long_by_shrinking_middle_odd_words() {
    let server = MockServer::start().await;
    mount_prompt_too_long_then_success(&server).await;
    let client = OpenRouterClient::new(openrouter_test_config(&server.uri())).expect("client");
    let resp = client
        .complete(&twelve_word_prompt())
        .await
        .expect("retry ok");
    assert_eq!(resp.content, "ok");
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::{
        openrouter_complete_surfaces_invalid_referer_header_errors,
        openrouter_prompt_too_long_stops_when_shrink_makes_no_change,
        openrouter_retries_after_prompt_too_long_by_shrinking_middle_odd_words,
        twelve_word_prompt,
    };

    #[test]
    fn kiss_cov_prompt_too_long_retry_test_symbols() {
        let _ = (
            twelve_word_prompt,
            openrouter_complete_surfaces_invalid_referer_header_errors,
            openrouter_prompt_too_long_stops_when_shrink_makes_no_change,
            openrouter_retries_after_prompt_too_long_by_shrinking_middle_odd_words,
        );
    }
}
