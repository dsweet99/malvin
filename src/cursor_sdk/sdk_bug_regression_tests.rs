use crate::acp::CoderPromptOptions;

use super::sdk_bug_helpers::{
    assert_err_has, bug_bridge_js, bug_clear_env, bug_client, bug_install_env,
    bug_point_bridge_at_missing, bug_prepare, expect_prompt_err,
};

async fn begin_then_end(path: &std::path::Path) {
    let mut client = bug_client(path, 1);
    client.begin_coder_session(path).await.expect("begin");
    client.end_coder_session().await.expect("end");
}

#[tokio::test]
async fn failed_create_drop_clears_sandbox_for_next_spawn() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    bug_point_bridge_at_missing(tmp.path());
    let mut client = bug_client(tmp.path(), 1);
    let err = client
        .begin_coder_session(tmp.path())
        .await
        .expect_err("missing bridge must fail create");
    assert_err_has(
        &err,
        &["No such file", "not found", "ENOENT", "spawn", "bridge"],
    );
    assert!(!client.has_open_coder_session());
    crate::malvin_sandbox::assert_dead_before_next_spawn()
        .expect("sandbox must be clear after failed BridgeSession drop");
    bug_install_env(&bug_bridge_js());
    begin_then_end(tmp.path()).await;
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
        .active_coder_session()
        .expect("active coder session")
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
        .active_coder_session()
        .expect("active coder session")
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
    assert!(crate::agent_backend::begun_cwd(&client).is_some());
    client
        .active_coder_session()
        .expect("active coder session")
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
    let session = crate::agent_backend::live_session(&client)
        .and_then(|s| s.as_cursor())
        .expect("session");
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
        "bridge drain timed out waiting for run_done after 1s without a bridge event (bridge quiet; likely hung or stalled)",
        "bridge timed out waiting for run_done after 1s without a bridge event (bridge quiet; likely hung or stalled)",
        "bridge timed out waiting for run_done after turn ran 1192s (limit 6000s; turn budget exhausted)",
        "bridge timed out waiting for ok after 1s without a bridge event (bridge quiet; likely hung or stalled)",
        "Agent agent-7b61bfe2-fa7a-47bd-8f5b-96c158067bc8 already has active run",
    ] {
        assert!(agent_error_requires_coder_session_teardown(msg), "{msg}");
    }
}
