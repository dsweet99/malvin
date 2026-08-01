//! Fast `LlmTransport` contract tests (wiremock `OpenRouter` + scripted Local).

use crate::llm_transport::{
    ChatMessage, ChatRole, LocalLlmTransport, LlmTransport, OpenRouterTransport, TransportError,
};
use crate::local_llm::LocalCompletionEngine;
use crate::openrouter_transport::test_support::openrouter_test_config;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

pub(super) fn user_msg(content: &str) -> ChatMessage {
    ChatMessage {
        role: ChatRole::User,
        content: content.into(),
    }
}

pub(super) fn openrouter_transport(base_url: &str) -> LlmTransport {
    let transport =
        OpenRouterTransport::new(openrouter_test_config(base_url)).expect("openrouter transport");
    LlmTransport::OpenRouter(transport)
}

pub(super) fn scripted_local_ok(content: &str) -> LlmTransport {
    LlmTransport::Local(LocalLlmTransport::new(LocalCompletionEngine::scripted_ok(
        "scripted",
        content,
    )))
}

pub(super) fn scripted_local_err(detail: &str) -> LlmTransport {
    LlmTransport::Local(LocalLlmTransport::new(LocalCompletionEngine::scripted_err(
        "scripted",
        detail,
    )))
}

async fn mount_json_ok(server: &MockServer, content: &str) {
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"content": content}}],
            "usage": {"total_tokens": 3}
        })))
        .mount(server)
        .await;
}

async fn mount_status(server: &MockServer, status: u16, body: &str) {
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(status).set_body_string(body))
        .mount(server)
        .await;
}

#[tokio::test]
pub(super) async fn llm_transport_openrouter_ensure_ready_key_present_and_absent() {
    crate::agent_backend::test_support::install_openrouter_test_key();
    let t = openrouter_transport("http://127.0.0.1:9");
    assert!(t.ensure_ready().is_ok());
    let saved = std::env::var_os("OPENROUTER_API_KEY");
    #[allow(unsafe_code)]
    unsafe {
        std::env::remove_var("OPENROUTER_API_KEY");
    }
    let err = t.ensure_ready().expect_err("missing key");
    assert!(matches!(err, TransportError::Unauthorized { .. }));
    #[allow(unsafe_code)]
    unsafe {
        match saved {
            Some(v) => std::env::set_var("OPENROUTER_API_KEY", v),
            None => std::env::remove_var("OPENROUTER_API_KEY"),
        }
    }
}

#[tokio::test]
pub(super) async fn llm_transport_openrouter_complete_ok_and_err() {
    let server = MockServer::start().await;
    mount_json_ok(&server, "transport-ok").await;
    let ok = openrouter_transport(&server.uri())
        .complete(&[user_msg("hi")])
        .await
        .expect("openrouter ok");
    assert_eq!(ok.content, "transport-ok");

    let err_server = MockServer::start().await;
    mount_status(&err_server, 401, "bad key").await;
    let err = openrouter_transport(&err_server.uri())
        .complete(&[user_msg("hi")])
        .await
        .expect_err("openrouter err");
    assert!(matches!(err, TransportError::Unauthorized { .. }));
}

#[tokio::test]
pub(super) async fn llm_transport_local_scripted_ensure_ready_ok_and_complete() {
    let ok_t = scripted_local_ok("local-ok");
    assert!(ok_t.ensure_ready().is_ok());
    let ok = ok_t.complete(&[user_msg("hi")]).await.expect("local ok");
    assert_eq!(ok.content, "local-ok");

    let err_t = scripted_local_err("engine boom");
    assert!(err_t.ensure_ready().is_ok());
    let err = err_t.complete(&[user_msg("hi")]).await.expect_err("local err");
    assert!(matches!(err, TransportError::Engine(ref m) if m.contains("engine boom")));
}

#[tokio::test]
pub(super) async fn llm_transport_parity_openrouter_mock_and_scripted_local() {
    let server = MockServer::start().await;
    mount_json_ok(&server, "same-content").await;
    let messages = [user_msg("parity")];
    let or_ok = openrouter_transport(&server.uri())
        .complete(&messages)
        .await
        .expect("or ok");
    let local_ok = scripted_local_ok("same-content")
        .complete(&messages)
        .await
        .expect("local ok");
    assert_eq!(or_ok.content, local_ok.content);

    let err_server = MockServer::start().await;
    mount_status(&err_server, 500, "boom").await;
    let or_err = openrouter_transport(&err_server.uri())
        .complete(&messages)
        .await
        .expect_err("or err");
    let local_err = scripted_local_err("boom")
        .complete(&messages)
        .await
        .expect_err("local err");
    assert!(matches!(or_err, TransportError::ServerError { .. }));
    assert!(matches!(local_err, TransportError::Engine(_)));
}

#[test]
pub(super) fn kiss_cov_llm_transport_contract_symbols() {
    let _ = (
        user_msg,
        openrouter_transport,
        scripted_local_ok,
        scripted_local_err,
        stringify!(llm_transport_openrouter_ensure_ready_key_present_and_absent),
        stringify!(llm_transport_openrouter_complete_ok_and_err),
        stringify!(llm_transport_local_scripted_ensure_ready_ok_and_complete),
        stringify!(llm_transport_parity_openrouter_mock_and_scripted_local),
    );
}
