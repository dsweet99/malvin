use std::path::Path;

use crate::acp::{
    outgoing_prompt_trace, AgentClient, AgentError, CoderSessionPromptDispatch,
};

use super::client_impl_prompt_retry::run_coder_prompt_with_retries;

impl AgentClient {
    /// Run one prompt on the open coder session (bug fix, summary, or learn).
    ///
    /// `who` names the outbound/inbound **trace stem** when `opts.do_trace_split` is `None` (for example
    /// `bug_fix` for `bug_fix.md`). `opts.stdout_bracket_label`
    /// overrides the stdout `[label...]` line and is usually the template filename (for example
    /// `bug_fix.md`); pass `None` to default the bracket label to `who`. When `do_trace_split` is `Some`,
    /// stems come from the split trace and `who` / `stdout_bracket_label`
    /// are not used for the split path.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when there is no session or the prompt fails after retries.
    pub async fn run_coder_prompt(
        &mut self,
        prompt: &str,
        log_path: &Path,
        who: &str,
        opts: outgoing_prompt_trace::CoderPromptOptions<'_>,
    ) -> Result<(), AgentError> {
        let outgoing_prompt_trace::CoderPromptOptions {
            llm_phase,
            do_trace_split,
            stdout_bracket_label,
            single_attempt,
        } = opts;
        let session = self
            .coder_session
            .as_ref()
            .ok_or_else(|| AgentError("begin_coder_session was not called".to_string()))?
            .clone();

        crate::prompts::enforce_no_unresolved_braces_in(prompt, stdout_bracket_label)
            .map_err(|e| AgentError(e.0))?;

        let dispatch = CoderSessionPromptDispatch {
            session: &session,
            full_prompt: prompt,
            log_path,
            who,
            do_trace_split,
            stdout_bracket_label,
        };
        run_coder_prompt_with_retries(self, dispatch, llm_phase, single_attempt).await
    }

    /// Shut down the **coder** session. Safe to call when no session is open.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when shutdown fails.
    pub async fn end_coder_session(&mut self) -> Result<(), AgentError> {
        if let Some(s) = self.coder_session.take() {
            s.shutdown().await.map_err(AgentError)?;
        }
        Ok(())
    }

    /// Full agent message text from the last completed coder prompt on the open session.
    #[must_use]
    pub fn last_coder_prompt_agent_response(&self) -> Option<String> {
        let session = self.coder_session.as_ref()?;
        Some(
            session
                .0
                .prompt_round_health
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .agent_response_text()
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::AgentClient;
    use crate::acp::test_captive_session::captive_cat_acp_session_for_tests;
    use crate::acp::{AgentIoOptions, outgoing_prompt_trace::CoderPromptOptions};
    use crate::support_paths::DEFAULT_MAX_ACP_RETRIES;

    #[tokio::test]
    async fn run_coder_prompt_rejects_unresolved_braces_before_retry() {
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
        let log = tmp.path().join("coder.log");
        let err = client
            .run_coder_prompt(
                "hello {{ unresolved",
                &log,
                "test",
                CoderPromptOptions {
                    llm_phase: None,
                    do_trace_split: None,
                    stdout_bracket_label: None,
                    single_attempt: true,
                },
            )
            .await
            .expect_err("unresolved braces");
        assert!(err.0.contains("{{"));
    }

    #[tokio::test]
    async fn end_coder_session_is_noop_without_open_session() {
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
        client.end_coder_session().await.expect("noop shutdown");
        assert!(client.coder_session.is_none());
    }

    #[test]
    fn last_coder_prompt_agent_response_without_session_is_none() {
        let client = AgentClient::with_max_acp_retries(
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
        assert!(client.last_coder_prompt_agent_response().is_none());
    }
}
