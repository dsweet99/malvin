#[cfg(test)]
mod begin_coder_session_guard_tests {
    use crate::acp::{AgentClient, AgentIoOptions};

    #[tokio::test]
    async fn second_begin_errors_when_coder_session_slot_occupied() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let cwd = tmp.path();
        let session = crate::acp::test_captive_session::captive_cat_acp_session_for_tests(cwd);
        let mut client = AgentClient::new(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        );
        client.coder_session = Some(session);
        let err = client
            .begin_coder_session(cwd)
            .await
            .expect_err("expected second begin to fail");
        assert_eq!(err.0, "coder ACP session is already open");
        client
            .end_coder_session()
            .await
            .expect("shutdown inert test session");
    }

    #[test]
    fn has_open_coder_session_false_until_begin() {
        let client = AgentClient::new(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        );
        assert!(!client.has_open_coder_session());
    }
}
