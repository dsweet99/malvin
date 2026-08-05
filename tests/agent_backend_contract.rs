//! Integration contract for [`malvin::agent_backend::AgentBackend`] (mock ACP spawn).
//!
//! Stays under the default unit timing budget; no network or GPU.

#![cfg(unix)]

mod common;

use common::agent_backend_helpers::{
    finish_lifecycle, mini_backend_with_llm, mini_done_wire, restore_acp_env, run_lifecycle,
    test_io, NO_REAL_AGENT,
};
use common::{acp_mock_js, cached_mock_executable, chunk_line};
use malvin::acp::{AgentClient, CoderPromptOptions};
use malvin::agent_backend::AgentBackend;
use malvin::llm_transport::{LocalLlmTransport, LlmTransport};
use malvin::local_llm::LocalCompletionEngine;
use malvin::mini_agent::{LlmBackend, MockScript, MockStep};

fn mini_mock_backend() -> AgentBackend {
    mini_backend_with_llm(
        LlmBackend::Mock(std::sync::Mutex::new(MockScript {
            responses: vec![MockStep::Ok(mini_done_wire())],
            call_count: 0,
        })),
        test_io(),
    )
}

fn acp_mock_backend(mock_bin: &std::path::Path) -> AgentBackend {
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var("MALVIN_AGENT_ACP_BIN", mock_bin);
        std::env::set_var(NO_REAL_AGENT, "1");
    }
    AgentBackend::Acp(AgentClient::with_max_acp_retries(
        "cursor:auto".into(),
        test_io(),
        1,
    ))
}

async fn assert_prompt_requires_begin(backend: &mut AgentBackend, log: &std::path::Path, label: &str) {
    assert!(!backend.has_open_coder_session(), "{label}: closed before begin");
    let err = backend
        .run_coder_prompt("x", log, "coder", CoderPromptOptions::default())
        .await
        .expect_err("prompt without begin");
    assert!(err.0.contains("begin_coder_session"), "{label}: {err:?}");
}

#[tokio::test]
async fn agent_backend_acp_mock_lifecycle_and_parity_with_mini() {
    let mock = cached_mock_executable(&acp_mock_js("", &chunk_line("ACP_DONE")));
    let old_bin = std::env::var_os("MALVIN_AGENT_ACP_BIN");
    let old_no_real = std::env::var_os(NO_REAL_AGENT);
    let tmp = tempfile::tempdir().expect("tempdir");
    let log = tmp.path().join("prompt.log");

    let mut mini = mini_mock_backend();
    assert_prompt_requires_begin(&mut mini, &log, "mini").await;
    run_lifecycle(&mut mini, tmp.path(), &log).await;
    finish_lifecycle(&mut mini, "MINI_DONE").await;

    let mut acp = acp_mock_backend(&mock);
    assert_prompt_requires_begin(&mut acp, &log, "acp").await;
    run_lifecycle(&mut acp, tmp.path(), &log).await;
    finish_lifecycle(&mut acp, "ACP_DONE").await;
    restore_acp_env(old_bin, old_no_real);
}

#[tokio::test]
async fn agent_backend_mini_local_transport_lifecycle() {
    let engine = LocalCompletionEngine::scripted_ok("scripted", mini_done_wire().content);
    let mut backend = mini_backend_with_llm(
        LlmBackend::Transport(LlmTransport::Local(LocalLlmTransport::new(engine))),
        test_io(),
    );
    let tmp = tempfile::tempdir().expect("tempdir");
    let log = tmp.path().join("prompt.log");
    backend.ensure_authenticated().expect("local auth");
    run_lifecycle(&mut backend, tmp.path(), &log).await;
    finish_lifecycle(&mut backend, "MINI_DONE").await;
}
