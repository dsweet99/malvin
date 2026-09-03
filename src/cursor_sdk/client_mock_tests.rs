use crate::acp::{AgentIoOptions, CoderPromptOptions};
use crate::cursor_sdk::CursorSdkClient;

pub(super) fn mock_io() -> AgentIoOptions {
    AgentIoOptions {
        force: true,
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

pub(super) fn install_mock_bridge_env(mock: &std::path::Path) {
    unsafe {
        std::env::set_var("MALVIN_CURSOR_SDK_BRIDGE", mock);
        std::env::set_var("CURSOR_API_KEY", "test-key");
        std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
    }
}

pub(super) fn clear_mock_bridge_env() {
    unsafe {
        std::env::remove_var("MALVIN_CURSOR_SDK_BRIDGE");
        std::env::remove_var("MOCK_BRIDGE_ONCE_DIR");
    }
}

pub(super) fn mock_client(run_dir: &std::path::Path) -> CursorSdkClient {
    let mut client = crate::cursor_sdk::cursor_sdk_client_from_raw("cursor:auto", mock_io(), 1);
    client.prompts_log_run_dir = Some(run_dir.to_path_buf());
    client
}

pub(super) async fn prompt_once(client: &mut CursorSdkClient, log: &std::path::Path) {
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
    let (steps, tokens_in, tokens_out, cache_read, cache_write) = {
        let g = timing.lock().unwrap();
        (
            g.steps,
            g.tokens_in,
            g.tokens_out,
            g.cache_read,
            g.cache_write,
        )
    };
    assert!(steps >= 1);
    assert_eq!(tokens_in, Some(11));
    assert_eq!(tokens_out, Some(7));
    assert_eq!(cache_read, Some(0));
    assert_eq!(cache_write, Some(0));
}

fn assert_session_timing_synced(client: &CursorSdkClient) {
    assert!(
        crate::agent_backend::live_session(client)
            .and_then(|s| s.timing.as_ref())
            .is_some()
    );
}

pub(super) fn mock_bridge_path() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/cursor_sdk/mock_bridge.js")
}

async fn run_prompt_and_assert_usage(
    client: &mut CursorSdkClient,
    tmp: &tempfile::TempDir,
    timing: &std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
) {
    prompt_once(client, &tmp.path().join("prompts.log")).await;
    assert_usage(timing);
}

#[tokio::test]
async fn cursor_sdk_client_mock_bridge_prompt_records_usage() {
    let _guard = crate::test_utils::test_env_lock();
    install_mock_bridge_env(&mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = mock_client(tmp.path());
    let timing = client.attach_run_timing_for_session();
    client.begin_coder_session(tmp.path()).await.expect("begin");
    run_prompt_and_assert_usage(&mut client, &tmp, &timing).await;
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("mock reply")
    );
    client.end_coder_session().await.expect("end");
    clear_mock_bridge_env();
}

#[tokio::test]
async fn cursor_sdk_client_mock_bridge_reuses_one_process_for_many_prompts() {
    let _guard = crate::test_utils::test_env_lock();
    install_mock_bridge_env(&mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = mock_client(tmp.path());
    let timing = client.attach_run_timing_for_session();
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let log = tmp.path().join("prompts.log");
    prompt_once(&mut client, &log).await;
    assert!(client.has_open_coder_session());
    prompt_once(&mut client, &log).await;
    assert!(client.has_open_coder_session());
    let (steps, tokens_in, tokens_out) = {
        let g = timing.lock().unwrap();
        (g.steps, g.tokens_in, g.tokens_out)
    };
    assert!(steps >= 2);
    assert_eq!(tokens_in, Some(22));
    assert_eq!(tokens_out, Some(14));
    client.end_coder_session().await.expect("end");
    assert!(!client.has_open_coder_session());
    clear_mock_bridge_env();
}

#[tokio::test]
async fn cursor_sdk_warm_start_attach_after_begin_records_usage() {
    let _guard = crate::test_utils::test_env_lock();
    install_mock_bridge_env(&mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = mock_client(tmp.path());
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let timing = client.attach_run_timing_for_session();
    run_prompt_and_assert_usage(&mut client, &tmp, &timing).await;
    client.set_run_timing(Some(std::sync::Arc::clone(&timing)));
    assert_session_timing_synced(&client);
    client.end_coder_session().await.expect("end");
    clear_mock_bridge_env();
}

async fn prompt_need_dm_with_capture(
    client: &mut CursorSdkClient,
    log: &std::path::Path,
) -> String {
    crate::output::set_do_dm_stdout_mode(true);
    crate::output::enable_stdout_capture();
    client
        .run_coder_prompt(
            "NEED_DM please",
            log,
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("prompt");
    let out = crate::output::take_captured_stdout();
    crate::output::set_do_dm_stdout_mode(false);
    out
}

fn assert_dm_hello(out: &str, client: &CursorSdkClient) {
    assert_eq!(out, "Hello.");
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("__MALVIN_DM_START__\nHello.\n__MALVIN_DM_END__")
    );
}

#[tokio::test]
async fn cursor_sdk_run_done_result_feeds_do_dm_stdout() {
    let _guard = crate::test_utils::test_env_lock();
    install_mock_bridge_env(&mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = mock_client(tmp.path());
    let _ = client.attach_run_timing_for_session();
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let out = prompt_need_dm_with_capture(&mut client, &tmp.path().join("prompts.log")).await;
    assert_dm_hello(&out, &client);
    client.end_coder_session().await.expect("end");
    clear_mock_bridge_env();
}

#[tokio::test]
async fn fresh_agent_on_retry_recreates_after_non_teardown_timeout() {
    let _guard = crate::test_utils::test_env_lock();
    install_mock_bridge_env(&mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let once_dir = tmp.path().join("once");
    std::fs::create_dir_all(&once_dir).expect("once dir");
    unsafe {
        std::env::set_var("MOCK_BRIDGE_ONCE_DIR", &once_dir);
    }
    let mut client = crate::cursor_sdk::cursor_sdk_client_from_raw("cursor:auto", mock_io(), 3);
    client.prompts_log_run_dir = Some(tmp.path().to_path_buf());
    let _ = client.attach_run_timing_for_session();
    client.begin_coder_session(tmp.path()).await.expect("begin");
    client
        .run_coder_prompt(
            "NON_TEARDOWN_TIMEOUT_ONCE please",
            &tmp.path().join("prompts.log"),
            "router_header",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                fresh_agent_on_retry: true,
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("header retry on fresh agent");
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
    client.end_coder_session().await.expect("end");
    clear_mock_bridge_env();
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

#[tokio::test]
async fn start_coder_session_sends_header_even_if_session_already_open() {
    let _guard = crate::test_utils::test_env_lock();
    install_mock_bridge_env(&mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = mock_client(tmp.path());
    client.io.log_full_outgoing_prompts = true;
    let _ = client.attach_run_timing_for_session();
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let log = tmp.path().join("prompts.log");
    client.bind_session_header(
        "__MALVIN_DM_START__\nbound-header".into(),
        log.clone(),
        "header.md",
    );
    let ensure = client.start_coder_session(tmp.path()).await.expect("start");
    assert!(!ensure.is_fresh(), "overlapping spawn must still be Reused");
    assert!(client.header_delivered);
    let text = std::fs::read_to_string(&log).expect("log");
    assert!(
        text.contains("bound-header"),
        "header.md must be sent after early spawn; got {text:?}"
    );
    client.end_coder_session().await.expect("end");
    clear_mock_bridge_env();
}
