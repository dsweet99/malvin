//! Full client lifecycle against `mock_bridge.js`.

use crate::acp::{AgentIoOptions, CoderPromptOptions};
use crate::cursor_sdk::CursorSdkClient;

fn mock_io() -> AgentIoOptions {
    AgentIoOptions {
        force: true,
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

fn install_mock_bridge_env(mock: &std::path::Path) {
    unsafe {
        std::env::set_var("MALVIN_CURSOR_SDK_BRIDGE", mock);
        std::env::set_var("CURSOR_API_KEY", "test-key");
        std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
    }
}

fn mock_client(run_dir: &std::path::Path) -> CursorSdkClient {
    let mut client = CursorSdkClient::with_max_retries("auto".into(), mock_io(), 1);
    client.prompts_log_run_dir = Some(run_dir.to_path_buf());
    client
}

async fn prompt_once(client: &mut CursorSdkClient, log: &std::path::Path) {
    client
        .run_coder_prompt(
            "hi",
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

fn assert_usage(timing: &std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>) {
    let (steps, tokens_in, tokens_out) = {
        let g = timing.lock().unwrap();
        (g.steps, g.tokens_in, g.tokens_out)
    };
    assert!(steps >= 1);
    assert_eq!(tokens_in, Some(11));
    assert_eq!(tokens_out, Some(7));
}

#[tokio::test]
async fn cursor_sdk_client_mock_bridge_prompt_records_usage() {
    let _guard = crate::test_utils::test_env_lock();
    let mock = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src/cursor_sdk/mock_bridge.js");
    install_mock_bridge_env(&mock);
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = mock_client(tmp.path());
    let timing = client.attach_run_timing_for_session();
    client.begin_coder_session(tmp.path()).await.expect("begin");
    prompt_once(&mut client, &tmp.path().join("prompts.log")).await;
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("mock reply")
    );
    assert_usage(&timing);
    client.end_coder_session().await.expect("end");
    unsafe {
        std::env::remove_var("MALVIN_CURSOR_SDK_BRIDGE");
    }
}
