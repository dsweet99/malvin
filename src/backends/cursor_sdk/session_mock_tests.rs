use super::client_mock_tests::{
    clear_mock_bridge_env, install_mock_bridge_env, mock_bridge_path, mock_client, prompt_once,
};

#[tokio::test]
async fn session_create_send_close_records_usage_and_closes() {
    let _guard = crate::test_support::test_utils::test_env_lock();
    install_mock_bridge_env(&mock_bridge_path());
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = mock_client(tmp.path());
    let timing = client.attach_run_timing_for_session();
    client
        .begin_coder_session(tmp.path())
        .await
        .expect("create session");
    prompt_once(&mut client, &tmp.path().join("prompts.log")).await;
    let (tokens_in, tokens_out) = {
        let guard = timing.lock().expect("timing lock");
        (guard.tokens_in, guard.tokens_out)
    };
    assert_eq!(
        tokens_in,
        Some(11),
        "send should record input tokens from the session's run_done usage"
    );
    assert_eq!(
        tokens_out,
        Some(7),
        "send should record output tokens from the session's run_done usage"
    );
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("mock reply"),
        "the session should keep the model reply text"
    );
    client.end_coder_session().await.expect("close session");
    assert!(
        !client.has_open_coder_session(),
        "close should drop the open coder session"
    );
    clear_mock_bridge_env();
}

#[tokio::test]
async fn streamed_usage_is_recorded_once_for_each_turn_ending() {
    let _guard = crate::test_support::test_utils::test_env_lock();
    for (ending, expected_in, expected_out, fails) in [
        ("STREAM_FATAL_ONLY", 70, 7, true),
        ("FATAL_THEN_RUN_DONE", 70, 7, true),
        ("CLOSE_STDOUT", 70, 7, true),
        ("NEVER_RUN_DONE", 70, 7, true),
        ("CANCELLED_RUN", 3, 1, true),
        ("EMPTY_RESULT_RUN_DONE", 70, 7, false),
        ("NORMAL", 11, 7, false),
    ] {
        assert_streamed_turn_usage(ending, (expected_in, expected_out), fails).await;
    }
}

async fn assert_streamed_turn_usage(ending: &str, expected: (u64, u64), fails: bool) {
    use super::sdk_bug_helpers::{bug_clear_env, bug_prepare, bug_set_drain_idle_timeout_ms};
    let tmp = bug_prepare();
    bug_set_drain_idle_timeout_ms(100);
    let mut client = mock_client(tmp.path());
    let timing = client.attach_run_timing_for_session();
    client.begin_coder_session(tmp.path()).await.expect("begin");
    let session = crate::backends::agent_backend::live_session(&client)
        .and_then(|s| s.as_cursor())
        .expect("session");
    let result = session.send_prompt(&format!("USAGE_BEFORE_{ending}")).await;
    assert_eq!(result.is_err(), fails, "{ending}: {result:?}");
    {
        let timing = timing.lock().unwrap();
        assert_eq!(timing.tokens_in, Some(expected.0), "{ending}");
        assert_eq!(timing.tokens_out, Some(expected.1), "{ending}");
        assert_eq!(timing.usage_tx_count, 1, "{ending}");
    }
    client.end_coder_session().await.expect("end");
    bug_clear_env();
}
