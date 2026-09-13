use crate::acp::{AgentIoOptions, CoderPromptOptions};

fn pi_mock_io() -> AgentIoOptions {
    AgentIoOptions {

        no_tee: true,
        raw_output: true,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        log_full_outgoing_prompts: false,
    }
}

fn pi_install_mock_env() {
    unsafe {
        std::env::set_var("OPENAI_API_KEY", "test-key");
        std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
    }
}

fn pi_clear_mock_env() {
    unsafe {
        std::env::remove_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV);
    }
}

fn pi_mock_client(run_dir: &std::path::Path) -> crate::agent_backend::SdkClient {
    let mut client = crate::pi_sdk::pi_sdk_client_from_raw("rpi:openai/gpt-4o", pi_mock_io(), 1);
    client.prompts_log_run_dir = Some(run_dir.to_path_buf());
    client
}

#[tokio::test]
async fn pi_sdk_client_mock_rpc_prompt_records_usage() {
    let _guard = crate::test_utils::test_env_lock();
    pi_install_mock_env();
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = pi_mock_client(tmp.path());
    let timing = client.attach_run_timing_for_session();
    client.begin_coder_session(tmp.path()).await.expect("begin");
    assert!(
        crate::agent_backend::live_session(&client)
            .and_then(|s| s.as_cursor())
            .is_none()
    );
    run_hello_prompt(&mut client, tmp.path()).await;
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("echo:hello")
    );
    assert_eq!(timing.lock().unwrap().tokens_in, Some(3));
    client.end_coder_session().await.expect("end");
    pi_clear_mock_env();
}

async fn run_hello_prompt(client: &mut crate::agent_backend::SdkClient, run_dir: &std::path::Path) {
    run_pi_prompt(client, run_dir, "hello").await;
}

async fn run_pi_prompt(
    client: &mut crate::agent_backend::SdkClient,
    run_dir: &std::path::Path,
    prompt: &str,
) {
    client
        .active_coder_session()
        .expect("active coder session")
        .run_coder_prompt(
            prompt,
            &run_dir.join("prompts.log"),
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("prompt");
}


#[tokio::test]
async fn pi_sdk_agent_end_before_ack_completes() {
    let _guard = crate::test_utils::test_env_lock();
    pi_install_mock_env();
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = pi_mock_client(tmp.path());
    client.begin_coder_session(tmp.path()).await.expect("begin");
    client
        .active_coder_session().expect("active coder session").run_coder_prompt(
            "AGENT_END_BEFORE_ACK please",
            &tmp.path().join("prompts.log"),
            "coder",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..CoderPromptOptions::default()
            },
        )
        .await
        .expect("early agent_end turn");
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("early-end")
    );
    client.end_coder_session().await.expect("end");
    pi_clear_mock_env();
}

#[tokio::test]
async fn pi_sdk_empty_assistant_result_clears_prior_response() {
    let _guard = crate::test_utils::test_env_lock();
    pi_install_mock_env();
    let tmp = tempfile::tempdir().expect("tmp");
    let mut client = pi_mock_client(tmp.path());
    client.begin_coder_session(tmp.path()).await.expect("begin");
    run_hello_prompt(&mut client, tmp.path()).await;
    assert_eq!(
        client.last_coder_prompt_agent_response().as_deref(),
        Some("echo:hello")
    );
    run_pi_prompt(&mut client, tmp.path(), "EMPTY_ASSISTANT_RESULT please").await;
    assert_eq!(
        client.last_coder_prompt_agent_response(),
        None,
        "missing agent_end text must not leave prior turn text"
    );
    client.end_coder_session().await.expect("end");
    pi_clear_mock_env();
}
