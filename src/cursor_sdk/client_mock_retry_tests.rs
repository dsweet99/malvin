use crate::acp::CoderPromptOptions;
use crate::cursor_sdk::CursorSdkClient;

use super::client_mock_tests::{
    clear_mock_bridge_env, install_mock_bridge_env, mock_bridge_path, mock_client, mock_io,
};

fn prepare_retry_test_client(tmp: &tempfile::TempDir, once_dir: &std::path::Path) -> CursorSdkClient {
    std::fs::create_dir_all(once_dir).expect("once dir");
    unsafe {
        std::env::set_var("MOCK_BRIDGE_ONCE_DIR", once_dir);
    }
    let mut client = crate::cursor_sdk::cursor_sdk_client_from_raw("cursor:auto", mock_io(), 3);
    client.prompts_log_run_dir = Some(tmp.path().to_path_buf());
    let _ = client.attach_run_timing_for_session();
    client
}

#[tokio::test]
async fn fresh_agent_on_retry_recreates_after_non_teardown_timeout() {
    let _guard = crate::test_utils::test_env_lock();
    install_mock_bridge_env(&mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let once_dir = tmp.path().join("once");
    let mut client = prepare_retry_test_client(&tmp, &once_dir);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    run_fresh_agent_retry_prompt(&mut client, tmp.path()).await;
    assert_fresh_agent_retry_boots(&once_dir);
    client.end_coder_session().await.expect("end");
    clear_mock_bridge_env();
}

async fn run_fresh_agent_retry_prompt(client: &mut CursorSdkClient, tmp_path: &std::path::Path) {
    client
        .active_coder_session().expect("active coder session").run_coder_prompt(
            "NON_TEARDOWN_TIMEOUT_ONCE please",
            &tmp_path.join("prompts.log"),
            "router_header",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                fresh_agent_on_retry: true,
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("header retry on fresh agent");
}

fn assert_fresh_agent_retry_boots(once_dir: &std::path::Path) {
    let boots = std::fs::read_to_string(once_dir.join("boots")).expect("boots log");
    let creates = boots.lines().filter(|l| *l == "create").count();
    let resumes = boots.lines().filter(|l| *l == "resume").count();
    assert!(
        creates >= 2,
        "fresh_agent_on_retry must create again after timeout; boots={boots:?}"
    );
    assert_eq!(
        resumes, 0,
        "fresh_agent_on_retry must not Cursor-resume the prior agent; boots={boots:?}"
    );
    assert!(
        once_dir.join("non_teardown_timeout_once").exists(),
        "mock must have injected the one-shot timeout"
    );
}

#[tokio::test]
async fn start_coder_session_requires_bound_header() {
    let _guard = crate::test_utils::test_env_lock();
    install_mock_bridge_env(&mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = mock_client(tmp.path());
    let err = client
        .start_coder_session(tmp.path())
        .await
        .expect_err("missing header");
    assert!(err.message.contains("bind_session_header"), "got {err:?}");
    clear_mock_bridge_env();
}

fn setup_open_session_with_header(client: &mut CursorSdkClient, log: &std::path::Path) {
    client.io.log_full_outgoing_prompts = true;
    let _ = client.attach_run_timing_for_session();
    client.bind_session_header(
        "__MALVIN_DM_START__\nbound-header".into(),
        log.to_path_buf(),
        "header.md",
    );
}

async fn run_and_assert_header_session(
    client: &mut CursorSdkClient,
    dir: &std::path::Path,
    log: &std::path::Path,
) {
    client.begin_coder_session(dir).await.expect("begin");
    let ensure = client.start_coder_session(dir).await.expect("start");
    assert!(!ensure.is_fresh(), "overlapping spawn must still be Reused");
    assert!(client.header_lifecycle.is_satisfied());
    assert_header_text_in_log(log);
}

#[tokio::test]
async fn start_coder_session_sends_header_even_if_session_already_open() {
    let _guard = crate::test_utils::test_env_lock();
    install_mock_bridge_env(&mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = mock_client(tmp.path());
    let log = tmp.path().join("prompts.log");
    setup_open_session_with_header(&mut client, &log);
    run_and_assert_header_session(&mut client, tmp.path(), &log).await;
    client.end_coder_session().await.expect("end");
    clear_mock_bridge_env();
}

fn assert_header_text_in_log(log: &std::path::Path) {
    let text = std::fs::read_to_string(log).expect("log");
    assert!(
        text.contains("bound-header"),
        "header.md must be sent after early spawn; got {text:?}"
    );
}
