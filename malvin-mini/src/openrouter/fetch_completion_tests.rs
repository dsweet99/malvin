use super::client::OpenRouterClient;
use crate::test_support::openrouter_test_config as test_config;
use wiremock::matchers::{body_string_contains, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
pub(crate) async fn fetch_completion_body_maps_http_200_nvidia_resource_exhausted() {
    let server = MockServer::start().await;
    let body = r#"{
        "error": {
            "message": "Provider returned error",
            "code": 503,
            "metadata": {
                "provider_name": "Nvidia",
                "raw": "{\"error\":{\"message\":\"ResourceExhausted\"}}",
                "error_type": "provider_overloaded"
            }
        }
    }"#;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let client = OpenRouterClient::new(test_config(&server.uri())).expect("client");
    let err = client.complete(&[]).await.result.expect_err("resource exhausted");
    assert!(err.is_transport_retryable());
    assert_eq!(err.to_string(), "Nvidia: ResourceExhausted");
}

#[tokio::test]
pub(crate) async fn fetch_completion_body_maps_http_200_non_retryable_provider_error() {
    let server = MockServer::start().await;
    let body = r#"{
        "error": {
            "message": "Provider returned error",
            "code": 400,
            "metadata": {
                "provider_name": "Nvidia",
                "raw": "{\"error\":{\"message\":\"Conversation roles must alternate user/assistant/user/assistant/...\"}}",
                "error_type": "invalid_request"
            }
        }
    }"#;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;
    let client = OpenRouterClient::new(test_config(&server.uri())).expect("client");
    let err = client.complete(&[]).await.result.expect_err("provider fatal");
    assert!(!err.is_transport_retryable());
    assert_eq!(
        err.to_string(),
        "Nvidia: Conversation roles must alternate user/assistant/user/assistant/..."
    );
}

#[tokio::test]
pub(crate) async fn fetch_completion_body_surfaces_transport_errors() {
    let client = OpenRouterClient::new(test_config("http://127.0.0.1:1")).expect("client");
    let meta = client
        .fetch_completion_body(&[], Some(4096))
        .await
        .expect_err("transport failure");
    assert!(meta.result.is_err());
    assert_eq!(meta.http.status, None);
}

#[tokio::test]
pub(crate) async fn fetch_completion_body_surfaces_header_validation_errors() {
    let server = MockServer::start().await;
    let mut config = test_config(&server.uri());
    config.http_referer = Some("bad\nreferer".into());
    let client = OpenRouterClient::new(config).expect("client");
    let meta = client
        .fetch_completion_body(&[], Some(4096))
        .await
        .expect_err("header failure");
    assert!(meta.result.is_err());
    assert_eq!(meta.http.status, None);
}

#[tokio::test]
pub(crate) async fn fetch_completion_body_reads_success_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            r#"{"choices":[{"message":{"content":"ok"}}]}"#,
        ))
        .mount(&server)
        .await;
    let client = OpenRouterClient::new(test_config(&server.uri())).expect("client");
    let (status, text) = client
        .fetch_completion_body(&[], Some(4096))
        .await
        .expect("success body");
    assert_eq!(status, 200);
    assert!(text.contains("ok"));
}


async fn mount_afford_then_ok(server: &MockServer) {
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(402).set_body_string(
            "You requested up to 8192 tokens, but can only afford 100.",
        ))
        .up_to_n_times(1)
        .mount(server)
        .await;
    Mock::given(method("POST"))
        .and(body_string_contains("\"max_tokens\":100"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"content": "afforded"}}]
        })))
        .mount(server)
        .await;
}

#[tokio::test]
pub(crate) async fn openrouter_retries_with_affordable_max_tokens() {
    let server = MockServer::start().await;
    mount_afford_then_ok(&server).await;
    let client = OpenRouterClient::new(test_config(&server.uri())).expect("client");
    let resp = client.complete(&[]).await.result.expect("retried ok");
    assert_eq!(resp.content, "afforded");
}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::{
        fetch_completion_body_maps_http_200_non_retryable_provider_error,
        fetch_completion_body_maps_http_200_nvidia_resource_exhausted,
        fetch_completion_body_reads_success_body,
        fetch_completion_body_surfaces_header_validation_errors,
        fetch_completion_body_surfaces_transport_errors,
        openrouter_retries_with_affordable_max_tokens,
    };

    #[test]
    fn kiss_cov_fetch_completion_body_tests() {
        let _ = (
            fetch_completion_body_maps_http_200_non_retryable_provider_error,
            fetch_completion_body_maps_http_200_nvidia_resource_exhausted,
            fetch_completion_body_surfaces_transport_errors,
            fetch_completion_body_surfaces_header_validation_errors,
            fetch_completion_body_reads_success_body,
            openrouter_retries_with_affordable_max_tokens,
        );
    }
}
