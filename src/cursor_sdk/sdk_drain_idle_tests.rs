
use crate::acp::CoderPromptOptions;

use super::sdk_bug_helpers::{
    assert_err_has, bug_clear_env, bug_client, bug_prepare, bug_set_drain_idle_timeout_ms,
    expect_prompt_err,
};

#[tokio::test]
async fn never_run_done_idle_timeout_tears_down_and_retries() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    bug_set_drain_idle_timeout_ms(200);
    let mut client = bug_client(tmp.path(), 2);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let log = tmp.path().join("prompts.log");
    let err = expect_prompt_err(&mut client, "NEVER_RUN_DONE please", &log).await;
    assert_err_has(&err, &["bridge timed out", "run_done"]);
    assert!(!client.has_open_coder_session());
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
        .expect("retry after drain idle timeout");
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("mock reply")
    );
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[tokio::test]
async fn long_idle_never_run_done_still_blocked_at_800ms() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    bug_set_drain_idle_timeout_ms(5000);
    let mut client = bug_client(tmp.path(), 1);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let session = client.session.as_ref().expect("session");
    let raced = tokio::time::timeout(
        std::time::Duration::from_millis(800),
        session.send_prompt("NEVER_RUN_DONE please"),
    )
    .await;
    assert!(
        raced.is_err(),
        "drain must still be blocked at 800ms when idle is 5s"
    );
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[tokio::test]
async fn keep_alive_events_do_not_trip_idle_drain_timeout() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    bug_set_drain_idle_timeout_ms(200);
    let mut client = bug_client(tmp.path(), 1);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let log = tmp.path().join("prompts.log");
    client
        .run_coder_prompt(
            "KEEP_ALIVE_THEN_DONE please",
            &log,
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("keep-alive turn must complete");
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("kept-alive")
    );
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}

#[test]
fn kiss_cov_sdk_drain_idle_cases() {
    let _ = stringify!(never_run_done_idle_timeout_tears_down_and_retries);
    let _ = stringify!(long_idle_never_run_done_still_blocked_at_800ms);
    let _ = stringify!(keep_alive_events_do_not_trip_idle_drain_timeout);
    let _ = stringify!(create_ack_idle_timeout_fails_begin);
    let _ = stringify!(empty_result_run_done_clears_prior_last_response);
}

#[tokio::test]
async fn create_ack_idle_timeout_fails_begin() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    bug_set_drain_idle_timeout_ms(200);
    unsafe {
        std::env::set_var("MOCK_BRIDGE_HANG_CREATE", "1");
    }
    let mut client = bug_client(tmp.path(), 1);
    let err = client
        .begin_coder_session(tmp.path())
        .await
        .expect_err("hung create must time out");
    assert_err_has(&err, &["bridge timed out", "ok"]);
    unsafe {
        std::env::remove_var("MOCK_BRIDGE_HANG_CREATE");
    }
    bug_clear_env();
}

#[tokio::test]
async fn empty_result_run_done_clears_prior_last_response() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    let mut client = bug_client(tmp.path(), 1);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let log = tmp.path().join("prompts.log");
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
        .expect("first turn");
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("mock reply")
    );
    client
        .run_coder_prompt(
            "EMPTY_RESULT_RUN_DONE please",
            &log,
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("empty-result turn");
    assert_eq!(
        client.last_coder_prompt_agent_response(),
        None,
        "missing RunDone.result must not leave prior turn text"
    );
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}
