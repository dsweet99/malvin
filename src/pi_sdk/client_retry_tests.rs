use std::sync::atomic::Ordering;

use crate::acp::{AgentError, AgentIoOptions, CoderPromptOptions};

use super::session_fake::FAKE_BACKEND_ERROR_TURNS;

const fn retry_mock_io() -> AgentIoOptions {
    AgentIoOptions {
        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

struct RetryMockEnv {
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl RetryMockEnv {
    fn install() -> Self {
        let guard = crate::test_utils::test_env_lock();
        unsafe {
            std::env::set_var("OPENAI_API_KEY", "test-key");
            std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
        }
        FAKE_BACKEND_ERROR_TURNS.store(0, Ordering::SeqCst);
        Self { _guard: guard }
    }
}

impl Drop for RetryMockEnv {
    fn drop(&mut self) {
        unsafe {
            std::env::remove_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV);
        }
    }
}

fn retry_mock_client(
    run_dir: &std::path::Path,
    max_retries: u32,
) -> crate::agent_backend::SdkClient {
    let mut client =
        crate::pi_sdk::pi_sdk_client_from_raw("rpi:openai/gpt-4o", retry_mock_io(), max_retries);
    client.prompts_log_run_dir = Some(run_dir.to_path_buf());
    client
}

async fn run_failing_prompt(
    client: &mut crate::agent_backend::SdkClient,
    run_dir: &std::path::Path,
    prompt: &str,
) -> AgentError {
    client
        .active_coder_session()
        .expect("active coder session")
        .run_coder_prompt(
            prompt,
            &run_dir.join("prompts.log"),
            "coder",
            CoderPromptOptions {
                fresh_agent_on_retry: true,
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect_err("persistent backend error must fail the prompt")
}

fn fake_error_turns() -> u32 {
    FAKE_BACKEND_ERROR_TURNS.load(Ordering::SeqCst)
}

#[tokio::test]
async fn persistent_prompt_error_stops_after_max_retries_despite_respawn() {
    let _env = RetryMockEnv::install();
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = retry_mock_client(tmp.path(), 3);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let err = run_failing_prompt(&mut client, tmp.path(), "PERSISTENT_BACKEND_ERROR").await;
    assert_eq!(fake_error_turns(), 3, "error: {err}");
    assert!(
        err.message.contains("repeated 3 times in a row"),
        "error: {err}"
    );
    let _ = client.end_coder_session().await;
}

#[tokio::test]
async fn persistent_header_error_stops_after_max_retries_despite_respawn() {
    let _env = RetryMockEnv::install();
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = retry_mock_client(tmp.path(), 3);
    client.bind_session_header(
        "PERSISTENT_BACKEND_ERROR header".into(),
        tmp.path().join("prompts.log"),
        "header.md",
    );
    let err = client
        .start_coder_session(tmp.path())
        .await
        .expect_err("persistent header error must fail the session start");
    assert_eq!(fake_error_turns(), 3, "error: {err}");
    assert!(
        err.message.contains("repeated 3 times in a row"),
        "error: {err}"
    );
    let _ = client.end_coder_session().await;
}

#[tokio::test]
async fn persistent_header_error_inside_prompt_retry_stops_after_max_retries() {
    let _env = RetryMockEnv::install();
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = retry_mock_client(tmp.path(), 3);
    client.bind_session_header(
        "PERSISTENT_BACKEND_ERROR header".into(),
        tmp.path().join("prompts.log"),
        "header.md",
    );
    client.begin_coder_session(tmp.path()).await.expect("begin");
    client.end_coder_session().await.expect("end");
    let err = run_failing_prompt(&mut client, tmp.path(), "hello").await;
    assert_eq!(fake_error_turns(), 3, "error: {err}");
    assert!(
        err.message.contains("repeated 3 times in a row"),
        "error: {err}"
    );
    let _ = client.end_coder_session().await;
}

#[test]
fn header_error_then_persistent_respawn_failure_stops() {
    let _env = RetryMockEnv::install();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime");
        let message = rt.block_on(async {
            let tmp = tempfile::tempdir().expect("tmp");
            let mut client = retry_mock_client(tmp.path(), 3);
            client.bind_session_header(
                "PERSISTENT_BACKEND_ERROR header".into(),
                tmp.path().join("prompts.log"),
                "header.md",
            );
            client.begin_coder_session(tmp.path()).await.expect("begin");
            client.model.slug = "no-provider".into();
            let err = client
                .start_coder_session(tmp.path())
                .await
                .expect_err("respawn failure must stop the header loop");
            err.message
        });
        let _ = tx.send(message);
    });
    let message = rx
        .recv_timeout(std::time::Duration::from_secs(4))
        .expect("header loop must stop instead of retrying without limit");
    assert!(message.contains("repeated 3 times in a row"), "{message}");
    assert!(message.contains("rpi model id must be"), "{message}");
}

#[tokio::test]
async fn successful_prompt_resets_backend_error_streak() {
    let _env = RetryMockEnv::install();
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = retry_mock_client(tmp.path(), 3);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    assert!(!client.record_backend_error("Compute error"));
    assert!(!client.record_backend_error("Compute error"));
    client
        .active_coder_session()
        .expect("active coder session")
        .run_coder_prompt(
            "hello",
            &tmp.path().join("prompts.log"),
            "coder",
            CoderPromptOptions::default(),
        )
        .await
        .expect("prompt");
    assert_eq!(client.backend_error_tracker().consecutive_count(), 0);
    let _ = client.end_coder_session().await;
}

#[tokio::test]
async fn spawn_success_keeps_backend_error_streak() {
    let _env = RetryMockEnv::install();
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = retry_mock_client(tmp.path(), 3);
    assert!(!client.record_backend_error("Compute error"));
    client.begin_coder_session(tmp.path()).await.expect("begin");
    assert_eq!(client.backend_error_tracker().consecutive_count(), 1);
    let _ = client.end_coder_session().await;
}
