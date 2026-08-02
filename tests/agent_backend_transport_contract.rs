//! Mini through `LlmTransport` + Mini↔ACP user-visible lookalike via `AgentBackend`.
//!
//! Stays under the default unit timing budget; wiremock / scripted Local only.

#![cfg(unix)]

mod common;

use std::time::Duration;

use common::agent_backend_helpers::{
    finish_lifecycle, mini_backend_with_llm, mini_done_wire, restore_acp_env, run_lifecycle,
    tee_io, test_io, NO_REAL_AGENT,
};
use common::observability_parity::{
    assert_acp_trace_schema, assert_stdout_lacks_substring, trace_contains_substring,
};
use common::{acp_mock_js, cached_mock_executable, chunk_line};
use malvin::acp::AgentClient;
use malvin::agent_backend::AgentBackend;
use malvin::llm_transport::{LlmTransport, OpenRouterTransport};
use malvin::mini_agent::{LlmBackend, MockScript, MockStep};
use malvin::openrouter_transport::OpenRouterConfig;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

fn mini_mock_tee_backend() -> AgentBackend {
    mini_backend_with_llm(
        LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![MockStep::Ok(mini_done_wire())],
            call_count: 0,
        })),
        tee_io(),
    )
}

fn openrouter_config(base_url: &str) -> OpenRouterConfig {
    OpenRouterConfig {
        model: "anthropic/claude-sonnet-4".into(),
        api_key: "sk-test".into(),
        http_referer: Some("https://malvin.test".into()),
        request_timeout: Duration::from_secs(5),
        base_url: base_url.into(),
        max_tokens: Some(256),
    }
}

async fn mount_mini_done_json(server: &MockServer) {
    let content = mini_done_wire().content;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "choices": [{"message": {"content": content}}],
            "usage": {"total_tokens": 3}
        })))
        .mount(server)
        .await;
}

struct OpenRouterTestKeyGuard(Option<std::ffi::OsString>);

impl OpenRouterTestKeyGuard {
    fn install() -> Self {
        let saved = std::env::var_os("OPENROUTER_API_KEY");
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var("OPENROUTER_API_KEY", "sk-test");
        }
        Self(saved)
    }
}

impl Drop for OpenRouterTestKeyGuard {
    fn drop(&mut self) {
        #[allow(unsafe_code)]
        unsafe {
            match self.0.take() {
                Some(v) => std::env::set_var("OPENROUTER_API_KEY", v),
                None => std::env::remove_var("OPENROUTER_API_KEY"),
            }
        }
    }
}

fn mini_openrouter_backend(server: &MockServer) -> AgentBackend {
    let transport = OpenRouterTransport::new(openrouter_config(&server.uri())).expect("transport");
    mini_backend_with_llm(
        LlmBackend::Transport(LlmTransport::OpenRouter(transport)),
        test_io(),
    )
}

fn acp_mock_tee_backend(mock_bin: &std::path::Path) -> AgentBackend {
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_AGENT_ACP_BIN", mock_bin);
        std::env::set_var(NO_REAL_AGENT, "1");
    }
    AgentBackend::Acp(AgentClient::with_max_acp_retries(
        "cursor:auto".into(),
        tee_io(),
        1,
    ))
}

fn assert_narrative_tee(stdout: &std::path::Path, run: &std::path::Path, marker: &str) {
    let trace = run.join("trace.jsonl");
    assert_acp_trace_schema(&trace);
    trace_contains_substring(&trace, "agent_message_chunk");
    let out = std::fs::read_to_string(stdout).unwrap_or_default();
    assert!(out.contains(marker), "stdout should narrative-tee done text; got:\n{out}");
    assert_stdout_lacks_substring(stdout, "\"direction\"");
    assert_stdout_lacks_substring(stdout, "miniUsage");
}

async fn tee_lifecycle_and_assert(backend: &mut AgentBackend, run: &std::path::Path, marker: &str) {
    let stdout = run.join("stdout.log");
    let prompt = run.join("prompt.log");
    backend.set_prompts_log_run_dir(Some(run.to_path_buf()));
    malvin::output::set_stdout_log_path(Some(stdout.clone()));
    run_lifecycle(backend, run, &prompt).await;
    finish_lifecycle(backend, marker).await;
    malvin::output::set_stdout_log_path(None);
    assert_narrative_tee(&stdout, run, marker);
}

async fn run_mini_openrouter_lifecycle(server: &MockServer) {
    mount_mini_done_json(server).await;
    let mut backend = mini_openrouter_backend(server);
    let tmp = tempfile::tempdir().expect("tempdir");
    let log = tmp.path().join("prompt.log");
    let _key = OpenRouterTestKeyGuard::install();
    backend.ensure_authenticated().expect("auth");
    run_lifecycle(&mut backend, tmp.path(), &log).await;
    finish_lifecycle(&mut backend, "MINI_DONE").await;
}

#[tokio::test]
async fn agent_backend_mini_openrouter_transport_lifecycle() {
    let server = MockServer::start().await;
    run_mini_openrouter_lifecycle(&server).await;
}

#[tokio::test]
async fn agent_backend_mini_acp_user_visible_lookalike() {
    let mock = cached_mock_executable(&acp_mock_js("", &chunk_line("MINI_DONE")));
    let old_bin = std::env::var_os("MALVIN_AGENT_ACP_BIN");
    let old_no_real = std::env::var_os(NO_REAL_AGENT);
    let mini_run = tempfile::tempdir().expect("mini run");
    let acp_run = tempfile::tempdir().expect("acp run");

    let mut mini = mini_mock_tee_backend();
    tee_lifecycle_and_assert(&mut mini, mini_run.path(), "MINI_DONE").await;

    let mut acp = acp_mock_tee_backend(&mock);
    tee_lifecycle_and_assert(&mut acp, acp_run.path(), "MINI_DONE").await;
    restore_acp_env(old_bin, old_no_real);
}
