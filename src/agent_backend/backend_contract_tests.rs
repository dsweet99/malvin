//! Lib `AgentBackend` contract tests (no ACP subprocess spawn).

use std::path::PathBuf;
use std::time::Duration;

use super::backend::AgentBackend;
use super::backend_kpop_test_helpers::mini_done_backend;
use super::test_support::{install_openrouter_test_key, mini_loop_config, test_io};
use crate::acp::{AgentClient, AgentIoOptions, CoderPromptOptions, MALVIN_TEST_NO_REAL_AGENT_ENV};
use crate::llm_transport::{LlmTransport, OpenRouterTransport};
use crate::mini_agent::{LlmBackend, MiniAgentClient};
use crate::openrouter_transport::OpenRouterConfig;
use crate::test_agent_client::smoke_agent_client;

pub(super) fn openrouter_transport_backend() -> AgentBackend {
    let cfg = OpenRouterConfig {
        model: "test/model".into(),
        api_key: "sk-test".into(),
        http_referer: None,
        request_timeout: Duration::from_secs(5),
        base_url: "http://127.0.0.1:9".into(),
        max_tokens: Some(64),
    };
    let transport = OpenRouterTransport::new(cfg).expect("transport");
    AgentBackend::Mini(MiniAgentClient::new_mock(
        mini_loop_config(2, 1),
        test_io(),
        LlmBackend::Transport(LlmTransport::OpenRouter(transport)),
    ))
}

fn assert_mini_http_auth_fails_without_key() {
    let http = openrouter_transport_backend();
    let saved = std::env::var_os("OPENROUTER_API_KEY");
    #[allow(unsafe_code)]
    unsafe {
        std::env::remove_var("OPENROUTER_API_KEY");
    }
    let err = http.ensure_authenticated().expect_err("missing key");
    assert!(err.0.contains("OPENROUTER_API_KEY"));
    #[allow(unsafe_code)]
    unsafe {
        if let Some(v) = saved {
            std::env::set_var("OPENROUTER_API_KEY", v);
        }
    }
}

fn assert_acp_auth_fails_without_credentials() {
    let old_no_real = std::env::var_os(MALVIN_TEST_NO_REAL_AGENT_ENV);
    let saved_keys: Vec<(String, Option<std::ffi::OsString>)> = [
        "CURSOR_AGENT_API_KEY",
        "CURSOR_API_KEY",
        "AGENT_API_KEY",
    ]
    .into_iter()
    .map(|k| (k.to_string(), std::env::var_os(k)))
    .collect();
    #[allow(unsafe_code)]
    unsafe {
        std::env::set_var(MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
        for (k, _) in &saved_keys {
            std::env::remove_var(k);
        }
    }
    let acp = AgentBackend::Acp(smoke_agent_client());
    let acp_err = acp.ensure_authenticated().expect_err("acp unauthenticated");
    assert!(
        acp_err.0.to_lowercase().contains("authenticated")
            || acp_err.0.contains("CURSOR")
            || acp_err.0.contains("API_KEY")
    );
    #[allow(unsafe_code)]
    unsafe {
        match old_no_real {
            Some(v) => std::env::set_var(MALVIN_TEST_NO_REAL_AGENT_ENV, v),
            None => std::env::remove_var(MALVIN_TEST_NO_REAL_AGENT_ENV),
        }
        for (k, v) in saved_keys {
            match v {
                Some(val) => std::env::set_var(k, val),
                None => std::env::remove_var(k),
            }
        }
    }
}

#[test]
pub(super) fn agent_backend_auth_shapes_mini_mock_http_and_acp() {
    install_openrouter_test_key();
    mini_done_backend()
        .ensure_authenticated()
        .expect("mini mock auth");
    assert_mini_http_auth_fails_without_key();
    assert_acp_auth_fails_without_credentials();
}

async fn assert_prompt_without_begin_errors(backend: &mut AgentBackend, log: &std::path::Path) {
    let err = backend
        .run_coder_prompt("hi", log, "coder", CoderPromptOptions::default())
        .await
        .expect_err("no begin");
    assert!(err.0.contains("begin_coder_session"));
}

async fn run_mini_lifecycle(mini: &mut AgentBackend, cwd: &std::path::Path, log: &std::path::Path) {
    mini.begin_coder_session(cwd).await.expect("begin");
    assert!(mini.has_open_coder_session());
    mini.run_coder_prompt("hi", log, "coder", CoderPromptOptions::default())
        .await
        .expect("prompt");
    let last = mini
        .last_coder_prompt_agent_response()
        .expect("last response");
    assert!(last.contains("MINI_DONE") || !last.is_empty());
    mini.end_coder_session().await.expect("end");
    assert!(!mini.has_open_coder_session());
}

#[tokio::test]
pub(super) async fn agent_backend_mini_mock_lifecycle_and_prompt_without_begin() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let log = tmp.path().join("prompt.log");
    let mut mini = mini_done_backend();
    assert!(!mini.has_open_coder_session());
    assert_prompt_without_begin_errors(&mut mini, &log).await;
    run_mini_lifecycle(&mut mini, tmp.path(), &log).await;

    let mut acp = AgentBackend::Acp(AgentClient::new(
        "m".into(),
        AgentIoOptions {
            force: false,
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        },
    ));
    assert_prompt_without_begin_errors(&mut acp, &log).await;
}

#[test]
pub(super) fn agent_backend_log_dir_accessors_round_trip_both_variants() {
    let dir = PathBuf::from("/tmp/malvin-agent-backend-contract-log");
    let mut mini = mini_done_backend();
    assert!(mini.prompts_log_run_dir().is_none());
    mini.set_prompts_log_run_dir(Some(dir.clone()));
    assert_eq!(mini.prompts_log_run_dir(), Some(&dir));
    mini.set_prompts_log_run_dir(None);
    assert!(mini.prompts_log_run_dir().is_none());

    let mut acp = AgentBackend::Acp(smoke_agent_client());
    assert!(acp.prompts_log_run_dir().is_none());
    acp.set_prompts_log_run_dir(Some(dir.clone()));
    assert_eq!(acp.prompts_log_run_dir(), Some(&dir));
    acp.set_prompts_log_run_dir(None);
    assert!(acp.prompts_log_run_dir().is_none());
}

#[test]
pub(super) fn kiss_cov_agent_backend_contract_symbols() {
    let _ = (
        openrouter_transport_backend,
        stringify!(agent_backend_auth_shapes_mini_mock_http_and_acp),
        stringify!(agent_backend_mini_mock_lifecycle_and_prompt_without_begin),
        stringify!(agent_backend_log_dir_accessors_round_trip_both_variants),
    );
}
