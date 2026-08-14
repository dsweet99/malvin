
use crate::acp::{AgentIoOptions, CoderPromptOptions};
use crate::cursor_sdk::CursorSdkClient;

pub(super) fn bug_mock_io_forced() -> AgentIoOptions {
    AgentIoOptions {
        force: true,
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

pub(super) fn bug_mock_io_noforce() -> AgentIoOptions {
    AgentIoOptions {
        force: false,
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

pub(super) fn bug_install_env(mock: &std::path::Path) {
    unsafe {
        std::env::set_var("MALVIN_CURSOR_SDK_BRIDGE", mock);
        std::env::set_var("CURSOR_API_KEY", "test-key");
        std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
    }
}

pub(super) fn bug_clear_env() {
    unsafe {
        std::env::remove_var("MALVIN_CURSOR_SDK_BRIDGE");
        std::env::remove_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS");
        std::env::remove_var("MOCK_BRIDGE_HANG_CREATE");
    }
}

pub(super) fn bug_set_drain_idle_timeout_ms(ms: u64) {
    unsafe {
        std::env::set_var("MALVIN_SDK_DRAIN_IDLE_TIMEOUT_MS", ms.to_string());
    }
}

pub(super) fn bug_bridge_js() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/cursor_sdk/mock_bridge.js")
}

pub(super) fn bug_client(run_dir: &std::path::Path, retries: u32) -> CursorSdkClient {
    let mut client =
        crate::cursor_sdk::cursor_sdk_client_from_raw("cursor:auto", bug_mock_io_forced(), retries);
    client.prompts_log_run_dir = Some(run_dir.to_path_buf());
    client
}

pub(super) fn bug_client_noforce(run_dir: &std::path::Path) -> CursorSdkClient {
    let mut client =
        crate::cursor_sdk::cursor_sdk_client_from_raw("cursor:auto", bug_mock_io_noforce(), 1);
    client.prompts_log_run_dir = Some(run_dir.to_path_buf());
    client
}

pub(super) fn bug_prepare() -> tempfile::TempDir {
    crate::test_utils::enable_test_fast_teardown();
    bug_install_env(&bug_bridge_js());
    let tmp = tempfile::tempdir().expect("tmp");
    crate::malvin_sandbox::clear_active_sandbox_session();
    tmp
}

pub(super) fn assert_err_has(err: &crate::acp::AgentError, needles: &[&str]) {
    assert!(
        needles.iter().any(|n| err.0.contains(n)),
        "unexpected: {}",
        err.0
    );
}

pub(super) async fn expect_prompt_err(
    client: &mut CursorSdkClient,
    prompt: &str,
    log: &std::path::Path,
) -> crate::acp::AgentError {
    client
        .run_coder_prompt(
            prompt,
            log,
            "coder",
            CoderPromptOptions {
                single_attempt: true,
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect_err("expected failure")
}

#[test]
fn kiss_cov_sdk_bug_helpers() {
    let _ = bug_mock_io_forced;
    let _ = bug_mock_io_noforce;
    let _ = bug_install_env;
    let _ = bug_clear_env;
    let _ = bug_set_drain_idle_timeout_ms;
    let _ = bug_bridge_js;
    let _ = bug_client;
    let _ = bug_client_noforce;
    let _ = bug_prepare;
    let _ = assert_err_has;
    let _ = expect_prompt_err;
}
