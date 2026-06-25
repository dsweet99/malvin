use crate::acp::{
    backoff_after_agent_failure, AgentClient, AgentError, AgentKpopMultiturnCtl, AcpSession,
    KpopFlowOnceArgs, retries_noun, run_kpop_flow_once,
    run_kpop_multiturn_once,
};

impl AgentClient {
    /// Standalone `KPop`: one ACP multiturn session without injected repo style.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when spawn or a prompt fails after retries.
    pub async fn run_kpop_flow(
        client: &mut Self,
        flow: &KpopFlowOnceArgs<'_>,
        session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
    ) -> Result<(), AgentError> {
        client.set_timing_implement_display_name("kpop");
        crate::agent_phase::enter_kpop();
        let mut last_error = String::new();

        let mut attempts_used = 0_u32;
        let max_attempts = client.max_acp_retries;
        for attempt in 1..=max_attempts {
            attempts_used = attempt;
            match run_kpop_flow_once(client, flow, session_dotfile_backups).await {
                Ok(()) => {
                    crate::agent_phase::leave_kpop();
                    return Ok(());
                }
                Err(e) => {
                    last_error = e.0;
                    if backoff_after_agent_failure(
                        client.timing.as_ref(),
                        &last_error,
                        attempt,
                        max_attempts,
                    )
                    .await?
                    {
                        break;
                    }
                }
            }
        }

        let retries = attempts_used.saturating_sub(1);
        crate::agent_phase::leave_kpop();
        let noun = retries_noun(retries);
        Err(AgentError(format!(
            "agent acp (kpop flow) failed after {retries} {noun}. Last error:\n{last_error}"
        )))
    }

    /// Multiturn `KPop`: one ACP session; each [`crate::kpop_progression::KpopMultiturnState::next_prompt`] issues another `prompt` until done.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when spawn or a prompt fails after retries.
    pub async fn run_kpop_multiturn(
        &mut self,
        mut ctl: AgentKpopMultiturnCtl<'_, '_>,
    ) -> Result<(), AgentError> {
        self.set_timing_implement_display_name("kpop");
        crate::agent_phase::enter_kpop();
        let mut last_error = String::new();

        let mut attempts_used = 0_u32;
        let max_attempts = self.max_acp_retries;
        for attempt in 1..=max_attempts {
            attempts_used = attempt;
            match run_kpop_multiturn_once(self, &mut ctl).await {
                Ok(()) => {
                    crate::agent_phase::leave_kpop();
                    return Ok(());
                }
                Err(e) => {
                    ctl.state.reset_for_transport_retry();
                    last_error = e.0;
                    if backoff_after_agent_failure(
                        self.timing.as_ref(),
                        &last_error,
                        attempt,
                        max_attempts,
                    )
                    .await?
                    {
                        break;
                    }
                }
            }
        }

        crate::agent_phase::leave_kpop();
        let retries = attempts_used.saturating_sub(1);
        let noun = retries_noun(retries);
        Err(AgentError(format!(
            "agent acp (kpop multiturn) failed after {retries} {noun}. Last error:\n{last_error}"
        )))
    }
}

#[doc(hidden)]
impl AgentClient {
    pub fn replace_coder_session_slot_for_tests(&mut self, session: AcpSession) {
        self.coder_session = Some(session);
    }
}

#[cfg(test)]
mod begin_coder_session_guard_tests {
    use crate::acp::AgentIoOptions;
    use super::AgentClient;

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

    #[tokio::test]
    async fn run_kpop_flow_fails_when_begin_session_errors() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let log = tmp.path().join("kpop.log");
        let prompts = ["probe"];
        let flow = crate::acp::KpopFlowOnceArgs {
            cwd: tmp.path(),
            kpop_prompts: &prompts,
            kpop_log: &log,
        };
        let backups = crate::artifacts::SessionDotfileBackups::from_parts(
            crate::artifacts::SessionDotfileParts {
                kissconfig: crate::session_dotfile_backup::DotfileBackupState::Missing,
                malvin_checks: crate::session_dotfile_backup::DotfileBackupState::Missing,
                kissignore: crate::session_dotfile_backup::DotfileBackupState::Missing,
                malvin_config: crate::session_dotfile_backup::DotfileBackupState::Missing,
                gitignore: crate::session_dotfile_backup::GitignoreBackup::Missing,
                malvin_config_workspace: crate::session_dotfile_backup::DotfileBackupState::Missing,
            },
        );
        let env_guard = crate::test_utils::SavedEnvVars::capture(&[
            crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV,
            "MALVIN_AGENT_ACP_BIN",
        ]);
        #[allow(unsafe_code)]
        unsafe {
            std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
            std::env::remove_var("MALVIN_AGENT_ACP_BIN");
        }
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
            1,
        );
        let err = AgentClient::run_kpop_flow(&mut client, &flow, &backups)
            .await
            .expect_err("spawn without mock should fail");
        assert!(err.0.contains("kpop flow"));
        drop(env_guard);
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
