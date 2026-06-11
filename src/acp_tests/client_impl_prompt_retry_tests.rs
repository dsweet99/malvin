use crate::acp::client_impl_prompt_retry::{
    run_coder_prompt_with_retries, run_one_coder_prompt_attempt,
    TEST_PROMPT_OK_WITHOUT_DISPATCH,
};
use crate::acp::test_captive_session::captive_cat_acp_session_for_tests;
use crate::acp::{AgentClient, AgentIoOptions, CoderSessionPromptDispatch};
use crate::support_paths::DEFAULT_MAX_ACP_RETRIES;

fn run_async_test(future: impl std::future::Future<Output = ()>) {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(future);
}

#[test]
fn run_coder_prompt_with_retries_succeeds_on_first_attempt() {
    run_async_test(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let mut client = AgentClient::with_max_acp_retries(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
            DEFAULT_MAX_ACP_RETRIES,
        );
        client.coder_session = Some(captive_cat_acp_session_for_tests(cwd));
        client.coder_session_cwd = Some(cwd.to_path_buf());
        let session = client.coder_session.as_ref().expect("session").clone();
        let log = tmp.path().join("coder.log");
        let dispatch = CoderSessionPromptDispatch {
            session: &session,
            full_prompt: TEST_PROMPT_OK_WITHOUT_DISPATCH,
            log_path: &log,
            who: "test",
            do_trace_split: None,
            stdout_bracket_label: None,
        };
        run_coder_prompt_with_retries(&mut client, dispatch, None, true)
            .await
            .expect("test prompt succeeds without dispatch");
    });
}

#[test]
fn run_coder_prompt_with_retries_errors_without_open_session() {
    run_async_test(async {
        let mut client = AgentClient::with_max_acp_retries(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
            DEFAULT_MAX_ACP_RETRIES,
        );
        let tmp = tempfile::tempdir().expect("tempdir");
        let log = tmp.path().join("coder.log");
        let dispatch = CoderSessionPromptDispatch {
            session: &captive_cat_acp_session_for_tests(tmp.path()),
            full_prompt: "ping",
            log_path: &log,
            who: "test",
            do_trace_split: None,
            stdout_bracket_label: None,
        };
        let err = run_coder_prompt_with_retries(&mut client, dispatch, None, true)
            .await
            .expect_err("no session");
        assert!(err.0.contains("begin_coder_session"));
    });
}

#[test]
fn run_coder_prompt_with_retries_sleeps_between_attempts_when_not_single() {
    run_async_test(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let mut client = AgentClient::with_max_acp_retries(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
            2,
        );
        client.coder_session = Some(captive_cat_acp_session_for_tests(cwd));
        client.coder_session_cwd = Some(cwd.to_path_buf());
        let session = client.coder_session.as_ref().expect("session").clone();
        let log = tmp.path().join("coder.log");
        let dispatch = CoderSessionPromptDispatch {
            session: &session,
            full_prompt: "ping",
            log_path: &log,
            who: "test",
            do_trace_split: None,
            stdout_bracket_label: None,
        };
        let t0 = std::time::Instant::now();
        let err = run_coder_prompt_with_retries(&mut client, dispatch, None, false)
            .await
            .expect_err("cat harness cannot satisfy ACP prompt RPC");
        assert!(err.0.contains("failed after 1 retry"));
        assert!(t0.elapsed() >= std::time::Duration::from_secs(1));
    });
}

#[test]
fn run_coder_prompt_with_retries_returns_exhausted_error_on_prompt_failure() {
    run_async_test(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let mut client = AgentClient::with_max_acp_retries(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
            DEFAULT_MAX_ACP_RETRIES,
        );
        client.coder_session = Some(captive_cat_acp_session_for_tests(cwd));
        client.coder_session_cwd = Some(cwd.to_path_buf());
        let session = client.coder_session.as_ref().expect("session").clone();
        let log = tmp.path().join("coder.log");
        let dispatch = CoderSessionPromptDispatch {
            session: &session,
            full_prompt: "ping",
            log_path: &log,
            who: "test",
            do_trace_split: None,
            stdout_bracket_label: None,
        };
        let err = run_coder_prompt_with_retries(&mut client, dispatch, None, true)
            .await
            .expect_err("cat harness cannot satisfy ACP prompt RPC");
        assert!(err.0.contains("failed after 0 retries"));
    });
}

#[test]
fn run_one_coder_prompt_attempt_invokes_prompt_on_open_session() {
    run_async_test(async {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let mut client = AgentClient::with_max_acp_retries(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
            DEFAULT_MAX_ACP_RETRIES,
        );
        client.coder_session = Some(captive_cat_acp_session_for_tests(cwd));
        client.coder_session_cwd = Some(cwd.to_path_buf());
        let session = client.coder_session.as_ref().expect("session").clone();
        let log = tmp.path().join("coder.log");
        let dispatch = CoderSessionPromptDispatch {
            session: &session,
            full_prompt: "ping",
            log_path: &log,
            who: "test",
            do_trace_split: None,
            stdout_bracket_label: None,
        };
        let err = run_one_coder_prompt_attempt(&mut client, cwd, &dispatch, None)
            .await
            .expect_err("cat harness cannot satisfy ACP prompt RPC");
        assert!(!err.is_empty());
    });
}
