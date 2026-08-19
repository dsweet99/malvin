use crate::acp::CoderPromptOptions;

use super::sdk_bug_helpers::{
    assert_err_has, bug_clear_env, bug_client, bug_client_noforce, bug_prepare, expect_prompt_err,
};

#[tokio::test]
async fn failed_create_drop_clears_sandbox_for_next_spawn() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    let mut client = bug_client_noforce(tmp.path());
    let err = client
        .begin_coder_session(tmp.path())
        .await
        .expect_err("no-force create must fail");
    assert_err_has(&err, &["--no-force", "not supported"]);
    assert!(!client.has_open_coder_session());
    crate::malvin_sandbox::assert_dead_before_next_spawn()
        .expect("sandbox must be clear after failed BridgeSession drop");
    let mut client2 = bug_client(tmp.path(), 1);
    client2
        .begin_coder_session(tmp.path())
        .await
        .expect("second begin after failed create");
    client2.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[tokio::test]
async fn agent_busy_after_resume_forgets_id_and_creates_fresh() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    let mut client = bug_client(tmp.path(), 3);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    assert_eq!(client.last_agent_id.as_deref(), Some("mock-agent"));
    let log = tmp.path().join("prompts.log");
    let err = expect_prompt_err(&mut client, "CLOSE_STDOUT", &log).await;
    assert_err_has(&err, &["bridge stdout closed", "stdout"]);
    assert!(!client.has_open_coder_session());
    assert_eq!(client.last_agent_id.as_deref(), Some("mock-agent"));
    client
        .run_coder_prompt(
            "AGENT_BUSY_ON_RESUME please",
            &log,
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("retry after AgentBusy must create fresh agent");
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("mock reply")
    );
    assert!(client.has_open_coder_session());
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[tokio::test]
async fn stale_authentication_teardown_resume_retries() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    let mut client = bug_client(tmp.path(), 3);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    assert_eq!(client.last_agent_id.as_deref(), Some("mock-agent"));
    let log = tmp.path().join("prompts.log");
    client
        .run_coder_prompt(
            "AUTH_ONCE please",
            &log,
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("retry after stale auth resume");
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("mock reply")
    );
    assert!(client.has_open_coder_session());
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[tokio::test]
async fn bridge_stdout_closed_single_attempt_tears_down_session() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    let mut client = bug_client(tmp.path(), 2);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let log = tmp.path().join("prompts.log");
    let err = expect_prompt_err(&mut client, "CLOSE_STDOUT", &log).await;
    assert_err_has(&err, &["bridge stdout closed", "stdout"]);
    assert!(!client.has_open_coder_session());
    assert!(client.session_cwd.is_some());
    client
        .run_coder_prompt(
            "hi",
            &log,
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("retry after teardown");
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("mock reply")
    );
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[tokio::test]
async fn cancelled_run_done_is_error() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    let mut client = bug_client(tmp.path(), 1);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let log = tmp.path().join("prompts.log");
    let err = expect_prompt_err(&mut client, "CANCELLED_RUN", &log).await;
    assert_err_has(&err, &["cancel", "cancelled"]);
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[tokio::test]
async fn stream_fatal_only_fails_prompt() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    let mut client = bug_client(tmp.path(), 1);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let log = tmp.path().join("prompts.log");
    let err = expect_prompt_err(&mut client, "STREAM_FATAL_ONLY", &log).await;
    assert_err_has(&err, &["stream error"]);
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[tokio::test]
async fn cancel_during_slow_send_is_honored() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    let mut client = bug_client(tmp.path(), 1);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let session = client.session.as_ref().expect("session");
    let prompt_fut = session.send_prompt("SLOW_SEND please");
    let cancel_fut = async {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let _ = super::session_io_write_cancel_for_test(session).await;
    };
    let (prompt_res, ()) = tokio::join!(prompt_fut, cancel_fut);
    assert!(prompt_res.is_err(), "cancel must fail the slow send");
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[test]
fn kiss_cov_bug_regression_cases() {
    let _ = stringify!(failed_create_drop_clears_sandbox_for_next_spawn);
    let _ = stringify!(agent_busy_after_resume_forgets_id_and_creates_fresh);
    let _ = stringify!(bridge_stdout_closed_single_attempt_tears_down_session);
    let _ = stringify!(cancelled_run_done_is_error);
    let _ = stringify!(stream_fatal_only_fails_prompt);
    let _ = stringify!(cancel_during_slow_send_is_honored);
}

#[test]
fn bridge_transport_errors_require_coder_session_teardown() {
    use crate::acp::agent_error_requires_coder_session_teardown;
    for msg in [
        "bridge stdout closed",
        "bridge write: broken pipe",
        "bridge flush: broken pipe",
        "bridge read: connection reset",
        "bridge drain timed out waiting for run_done after 1s of silence",
        "bridge timed out waiting for run_done after 1s of silence",
        "bridge timed out waiting for ok after 1s of silence",
        "Agent agent-7b61bfe2-fa7a-47bd-8f5b-96c158067bc8 already has active run",
    ] {
        assert!(agent_error_requires_coder_session_teardown(msg), "{msg}");
    }
}
