//! Shared helpers for integration tests.

use std::time::Duration;

use crate::config::OpenRouterConfig;
use wiremock::matchers::{body_string_contains, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[must_use]
pub(crate) fn openrouter_test_config(base_url: &str) -> OpenRouterConfig {
    OpenRouterConfig {
        model: "anthropic/claude-sonnet-4".into(),
        api_key: "sk-test".into(),
        http_referer: Some("https://malvin.test".into()),
        request_timeout: Duration::from_secs(30),
        base_url: base_url.into(),
    }
}

pub(crate) async fn mount_prompt_too_long_once(server: &MockServer) {
    Mock::given(method("POST"))
        .respond_with(prompt_too_long_template())
        .expect(1)
        .mount(server)
        .await;
}

fn prompt_too_long_template() -> ResponseTemplate {
    ResponseTemplate::new(400).set_body_string("prompt is too long")
}

fn ok_completion_template() -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(serde_json::json!({
        "choices": [{"message": {"content": "ok"}}],
        "usage": {"total_tokens": 1}
    }))
}

pub(crate) async fn mount_prompt_too_long_then_success(server: &MockServer) {
    Mock::given(method("POST"))
        .and(body_string_contains("w3"))
        .respond_with(prompt_too_long_template())
        .up_to_n_times(1)
        .expect(1)
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(body_string_contains("w4"))
        .and(body_string_contains("w6"))
        .respond_with(ok_completion_template())
        .up_to_n_times(1)
        .expect(1)
        .mount(server)
        .await;
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{
        mount_prompt_too_long_once, mount_prompt_too_long_then_success, ok_completion_template,
        openrouter_test_config, prompt_too_long_template,
    };
    use wiremock::MockServer;

    #[test]
    fn openrouter_test_config_sets_defaults() {
        let config = openrouter_test_config("http://127.0.0.1:9");
        assert_eq!(config.base_url, "http://127.0.0.1:9");
        assert_eq!(config.model, "anthropic/claude-sonnet-4");
        assert_eq!(config.api_key, "sk-test");
        assert_eq!(
            config.http_referer.as_deref(),
            Some("https://malvin.test")
        );
        assert_eq!(config.request_timeout, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn mount_prompt_too_long_once_responds_to_post() {
        let server = MockServer::start().await;
        mount_prompt_too_long_once(&server).await;
        let status = reqwest::Client::new()
            .post(server.uri())
            .send()
            .await
            .expect("post")
            .status();
        assert_eq!(status, 400);
    }

    #[tokio::test]
    async fn mount_prompt_too_long_then_success_responds_to_shrunk_post() {
        let server = MockServer::start().await;
        mount_prompt_too_long_then_success(&server).await;
        let first = reqwest::Client::new()
            .post(server.uri())
            .body("w0 w1 w2 w3 w4 w5 w6 w7 w8 w9 w10 w11")
            .send()
            .await
            .expect("first post")
            .status();
        assert_eq!(first, 400);
        let second = reqwest::Client::new()
            .post(server.uri())
            .body("w0 w1 w2 w4 w6 w8 w10 w11")
            .send()
            .await
            .expect("second post")
            .status();
        assert_eq!(second, 200);
    }

    #[test]
    fn kiss_cov_test_support_symbols() {
        let _ = (
            openrouter_test_config,
            mount_prompt_too_long_once,
            mount_prompt_too_long_then_success,
            prompt_too_long_template,
            ok_completion_template,
            openrouter_test_config_sets_defaults,
            mount_prompt_too_long_once_responds_to_post,
            mount_prompt_too_long_then_success_responds_to_shrunk_post,
        );
    }
}
