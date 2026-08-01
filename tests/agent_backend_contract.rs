//! Integration contract for [`malvin::agent_backend::AgentBackend`] (mock ACP spawn).
//!
//! Stays under the default unit timing budget; no network or GPU.

#![cfg(unix)]

mod common;

use common::{acp_mock_js, cached_mock_executable, chunk_line};
use malvin::acp::{AgentClient, AgentIoOptions, CoderPromptOptions, KpopFlowOnceArgs};
use malvin::agent_backend::{agent_backend_run_kpop_flow, AgentBackend};
use malvin::artifacts::{
    GitignoreBackup, MalvinChecksBackup, MalvinConfigBackup, MalvinConfigWorkspaceBackup,
    SessionDotfileBackups, SessionDotfileParts, VisionBackup,
};
use malvin::mini_agent::{LlmBackend, MiniAgentClient, MockScript, MockStep};
use malvin::openrouter_transport::CompletionResponse;

const NO_REAL_AGENT: &str = "MALVIN_TEST_NO_REAL_AGENT";

fn mini_done_wire() -> CompletionResponse {
    CompletionResponse {
        content: malvin::mini_agent::protocol::format_wire_turn("- done", "MINI_DONE"),
        usage: None,
        reasoning: None,
    }
}

fn empty_backups() -> SessionDotfileBackups {
    SessionDotfileBackups::from_parts(SessionDotfileParts {
        malvin_checks: MalvinChecksBackup::Missing,
        malvin_config: MalvinConfigBackup::Missing,
        gitignore: GitignoreBackup::Missing,
        vision: VisionBackup::Missing,
        malvin_config_workspace: MalvinConfigWorkspaceBackup::Missing,
    })
}

const fn test_io() -> AgentIoOptions {
    AgentIoOptions {
        force: false,
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

fn mini_mock_backend() -> AgentBackend {
    AgentBackend::Mini(
        MiniAgentClient::new(
            malvin::mini_agent::MiniLoopConfig {
                model: "anthropic/claude-sonnet-4".into(),
                max_http_turns: 4,
                max_bash_execs: 128,
                max_http_retries: 1,
                max_transport_retries: 1,
                max_gate_retries: 1,
                max_shrink_passes: 0,
                retry_strategy: malvin::mini_agent::MiniRetryStrategy::CumulativeTranscript,
                expects_investigation: false,
                allow_download: true,
                mini_constraints: "constraints".into(),
            },
            test_io(),
            LlmBackend::Mock(std::sync::Mutex::new(MockScript {
                responses: vec![MockStep::Ok(mini_done_wire())],
                call_count: 0,
            })),
        )
        .expect("mini client"),
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

fn restore_acp_env(old_bin: Option<std::ffi::OsString>, old_no_real: Option<std::ffi::OsString>) {
    #[allow(unsafe_code)]
    unsafe {
        match old_bin {
            Some(v) => std::env::set_var("MALVIN_AGENT_ACP_BIN", v),
            None => std::env::remove_var("MALVIN_AGENT_ACP_BIN"),
        }
        match old_no_real {
            Some(v) => std::env::set_var(NO_REAL_AGENT, v),
            None => std::env::remove_var(NO_REAL_AGENT),
        }
    }
}

async fn assert_prompt_requires_begin(backend: &mut AgentBackend, log: &std::path::Path, label: &str) {
    assert!(!backend.has_open_coder_session(), "{label}: closed before begin");
    let err = backend
        .run_coder_prompt("x", log, "coder", CoderPromptOptions::default())
        .await
        .expect_err("prompt without begin");
    assert!(err.0.contains("begin_coder_session"), "{label}: {err:?}");
}

async fn run_lifecycle(backend: &mut AgentBackend, cwd: &std::path::Path, log: &std::path::Path) {
    backend.begin_coder_session(cwd).await.expect("begin");
    assert!(backend.has_open_coder_session());
    backend
        .run_coder_prompt("hello", log, "coder", CoderPromptOptions::default())
        .await
        .expect("prompt");
}

async fn finish_lifecycle(backend: &mut AgentBackend, expect_marker: &str) {
    let last = backend
        .last_coder_prompt_agent_response()
        .expect("last");
    assert!(last.contains(expect_marker), "last={last}");
    backend.end_coder_session().await.expect("end");
    assert!(!backend.has_open_coder_session());
}

#[tokio::test]
async fn agent_backend_acp_mock_lifecycle_and_parity_with_mini() {
    let mock = cached_mock_executable(&acp_mock_js("", &chunk_line("ACP_DONE")));
    let old_bin = std::env::var_os("MALVIN_AGENT_ACP_BIN");
    let old_no_real = std::env::var_os(NO_REAL_AGENT);
    let tmp = tempfile::tempdir().expect("tempdir");
    let log = tmp.path().join("prompt.log");

    // Mini parity for prompt-without-begin / session flags (no ACP spawn).
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
async fn agent_backend_acp_kpop_flow_via_enum_with_mock_bin() {
    let mock = cached_mock_executable(&acp_mock_js("", &chunk_line("KPOP_OK")));
    let old_bin = std::env::var_os("MALVIN_AGENT_ACP_BIN");
    let old_no_real = std::env::var_os(NO_REAL_AGENT);
    let tmp = tempfile::tempdir().expect("tempdir");
    let log = tmp.path().join("kpop.log");
    let mut backend = acp_mock_backend(&mock);
    let prompts = ["kpop hello"];
    let args = KpopFlowOnceArgs {
        cwd: tmp.path(),
        kpop_prompts: &prompts,
        kpop_log: &log,
    };
    agent_backend_run_kpop_flow(&mut backend, &args, &empty_backups())
        .await
        .expect("acp kpop via AgentBackend");
    restore_acp_env(old_bin, old_no_real);
}
