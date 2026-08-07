//! Full client lifecycle against `mock_bridge.js`.

use crate::acp::{AgentIoOptions, CoderPromptOptions};
use crate::prime_sdk::PrimeSdkClient;

fn prime_mock_io() -> AgentIoOptions {
    AgentIoOptions {
        force: true,
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

fn prime_install_mock_bridge_env(mock: &std::path::Path) {
    unsafe {
        std::env::set_var("MALVIN_PRIME_SDK_BRIDGE", mock);
        std::env::set_var("OPENAI_API_KEY", "test-key");
        std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
    }
}

fn prime_clear_mock_bridge_env() {
    unsafe {
        std::env::remove_var("MALVIN_PRIME_SDK_BRIDGE");
    }
}

fn prime_mock_bridge_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/prime_sdk/mock_bridge.js")
}

fn prime_mock_client(run_dir: &std::path::Path) -> PrimeSdkClient {
    let mut client =
        PrimeSdkClient::with_max_retries("prime:openai/gpt-4o".into(), prime_mock_io(), 1);
    client.prompts_log_run_dir = Some(run_dir.to_path_buf());
    client
}

async fn prime_prompt_once(client: &mut PrimeSdkClient, log: &std::path::Path) {
    client
        .run_coder_prompt(
            "hello",
            log,
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("prompt");
}

#[tokio::test]
async fn prime_sdk_client_mock_bridge_prompt_records_usage() {
    let _guard = crate::test_utils::test_env_lock();
    prime_install_mock_bridge_env(&prime_mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = prime_mock_client(tmp.path());
    let timing = client.attach_run_timing_for_session();
    client.begin_coder_session(tmp.path()).await.expect("begin");
    prime_prompt_once(&mut client, &tmp.path().join("prompts.log")).await;
    assert!(client
        .last_coder_prompt_agent_response()
        .unwrap()
        .contains("echo:hello"));
    let (tokens_in, steps) = {
        let g = timing.lock().unwrap();
        (g.tokens_in, g.steps)
    };
    assert_eq!(tokens_in, Some(3));
    assert!(
        steps >= 1,
        "prime bridge must emit step events for COST parity with cursor:; steps={steps}"
    );
    client.end_coder_session().await.expect("end");
    prime_clear_mock_bridge_env();
}
