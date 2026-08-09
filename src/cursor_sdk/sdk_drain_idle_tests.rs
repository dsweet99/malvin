//! Idle drain-timeout regressions for Cursor SDK `drain_until_run_done`.

use crate::acp::CoderPromptOptions;

use super::sdk_bug_helpers::{
    assert_err_has, bug_clear_env, bug_client, bug_prepare, bug_set_drain_idle_timeout_ms,
    expect_prompt_err,
};

/// Missing `run_done`: idle drain timeout must fail the turn and tear down for retry.
#[tokio::test]
async fn never_run_done_idle_timeout_tears_down_and_retries() {
    let _guard = crate::test_utils::test_env_lock();
    let tmp = bug_prepare();
    bug_set_drain_idle_timeout_ms(200);
    let mut client = bug_client(tmp.path(), 2);
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let log = tmp.path().join("prompts.log");
    let err = expect_prompt_err(&mut client, "NEVER_RUN_DONE please", &log).await;
    assert_err_has(&err, &["bridge drain timed out", "run_done"]);
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

/// With a long idle budget, missing `run_done` still blocks past a short outer deadline.
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

/// Idle timeout is between events: keep-alive ticks longer than idle still complete.
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
}
