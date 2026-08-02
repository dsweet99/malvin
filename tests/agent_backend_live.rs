//! Opt-in live [`malvin::agent_backend::AgentBackend`] integration tests.
//!
//! ```text
//! MALVIN_LIVE_MINI=1 OPENROUTER_API_KEY=... cargo nextest run -E 'test(agent_backend_live)' -- --ignored
//! MALVIN_LIVE_LOCAL=1 cargo nextest run -E 'test(agent_backend_live)' -- --ignored
//! ```
//!
//! ACP live uses existing live-agent prereqs (no new `MALVIN_LIVE_AGENT`).
//! Local/GPU Mini+`local:` cases are Metal-only and stay disabled by default.

#![cfg(unix)]

mod common;

use common::{
    live_agent_prereqs_met, require_openrouter_key_when_gate_set, LIVE_AGENT_CMD_TIMEOUT,
};
use malvin::acp::{AgentClient, AgentIoOptions, CoderPromptOptions};
use malvin::agent_backend::{build_agent_backend, AgentBackend};
use malvin::cli::{SharedOpts, WorkflowCliOptions};

fn live_mini_gate_set() -> bool {
    std::env::var_os("MALVIN_LIVE_MINI").is_some_and(|v| v == "1")
}

fn live_local_gate_set() -> bool {
    std::env::var_os("MALVIN_LIVE_LOCAL").is_some_and(|v| v == "1")
}

async fn live_single_attempt_prompt(backend: &mut AgentBackend, cwd: &std::path::Path, log_name: &str) {
    backend.begin_coder_session(cwd).await.expect("begin");
    let log = cwd.join(log_name);
    backend
        .run_coder_prompt(
            "Reply with exactly: pong. Do not use bash.",
            &log,
            "coder",
            CoderPromptOptions {
                single_attempt: true,
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("prompt");
    assert!(backend
        .last_coder_prompt_agent_response()
        .is_some_and(|s| !s.trim().is_empty()));
    backend.end_coder_session().await.expect("end");
}

fn openrouter_shared_opts() -> SharedOpts {
    SharedOpts {
        model: "openrouter:auto".into(),
        no_force: false,
        no_tenacious: true,
        gates: false,
        quiet: false,
        verbose: false,
        max_acp_retries: 1,
        doc: false,
        name: None,
        mini_max_bash_turns: 32,
        mini_max_http_turns: 4,
        mini_max_bash_execs: 8,
        mini_max_http_retries: 1,
        mini_max_transport_retries: 1,
        mini_max_gate_retries: 1,
        mini_max_shrink_passes: 0,
        no_download: false,
        git: false,
    }
}

fn local_shared_opts() -> SharedOpts {
    let mut o = openrouter_shared_opts();
    o.model = "local:nemotron3_nano_4b".into();
    o
}

#[test]
fn agent_backend_live_tests_compile_and_skip_without_env() {
    let _ = (
        live_mini_gate_set(),
        live_local_gate_set(),
        live_agent_prereqs_met(),
        LIVE_AGENT_CMD_TIMEOUT,
    );
}

#[tokio::test]
#[ignore = "live Mini+OpenRouter via AgentBackend; MALVIN_LIVE_MINI=1 OPENROUTER_API_KEY=... cargo nextest run -E 'test(agent_backend_live)' -- --ignored"]
async fn agent_backend_live_mini_openrouter() {
    if !live_mini_gate_set() {
        return;
    }
    require_openrouter_key_when_gate_set("MALVIN_LIVE_MINI");
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut backend = build_agent_backend(
        &openrouter_shared_opts(),
        WorkflowCliOptions { force: false },
        false,
        "do",
    )
    .expect("mini backend");
    assert!(matches!(backend, AgentBackend::Mini(_)));
    backend.ensure_authenticated().expect("auth");
    live_single_attempt_prompt(&mut backend, tmp.path(), "live.log").await;
}

#[tokio::test]
#[ignore = "live ACP via AgentBackend; requires live cursor-agent auth (see tests/common/live_agent.rs)"]
async fn agent_backend_live_acp() {
    if !live_agent_prereqs_met() {
        eprintln!("skip: live cursor-agent prereqs not met");
        return;
    }
    #[allow(unsafe_code)]
    unsafe {
        std::env::remove_var("MALVIN_AGENT_ACP_BIN");
        std::env::remove_var("MALVIN_TEST_NO_REAL_AGENT");
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut backend = AgentBackend::Acp(AgentClient::with_max_acp_retries(
        "cursor:auto".into(),
        AgentIoOptions {
            force: false,
            no_tee: true,
            raw_output: true,
            show_thoughts_on_stdout: false,
            emit_stdout_markdown: false,
            log_full_outgoing_prompts: false,
        },
        1,
    ));
    backend.ensure_authenticated().expect("auth");
    backend
        .begin_coder_session(tmp.path())
        .await
        .expect("begin");
    let log = tmp.path().join("live_acp.log");
    backend
        .run_coder_prompt(
            "Reply with exactly: pong",
            &log,
            "coder",
            CoderPromptOptions {
                single_attempt: true,
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("prompt");
    assert!(backend
        .last_coder_prompt_agent_response()
        .is_some_and(|s| !s.trim().is_empty()));
    backend.end_coder_session().await.expect("end");
}

#[tokio::test]
#[ignore = "live Mini+local via AgentBackend (Metal); MALVIN_LIVE_LOCAL=1 cargo nextest run -E 'test(agent_backend_live)' -- --ignored"]
async fn agent_backend_live_mini_local() {
    if !live_local_gate_set() {
        return;
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut backend = build_agent_backend(
        &local_shared_opts(),
        WorkflowCliOptions { force: false },
        false,
        "do",
    )
    .expect("local mini backend");
    assert!(matches!(backend, AgentBackend::Mini(_)));
    live_single_attempt_prompt(&mut backend, tmp.path(), "live_local.log").await;
}
