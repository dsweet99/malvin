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
