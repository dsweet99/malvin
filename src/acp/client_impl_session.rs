use std::path::Path;

use crate::acp::{
    backoff_after_agent_failure, spawn_agent_acp_session, AgentClient, AgentError, AgentIoOptions,
    AuthError, retries_noun,
};
use crate::support_paths::DEFAULT_MAX_ACP_RETRIES;

impl AgentClient {
    #[must_use]
    pub const fn new(model: String, io: AgentIoOptions) -> Self {
        Self::with_max_acp_retries(model, io, DEFAULT_MAX_ACP_RETRIES)
    }

    #[must_use]
    pub const fn with_max_acp_retries(
        model: String,
        io: AgentIoOptions,
        max_acp_retries: u32,
    ) -> Self {
        Self {
            model,
            io,
            prompts_log_run_dir: None,
            coder_session: None,
            max_acp_retries: if max_acp_retries == 0 {
                1
            } else {
                max_acp_retries
            },
            timing: None,
        }
    }

    /// When set (orchestrator, standalone `KPop`), LLM waits and retry backoff are recorded.
    pub fn set_run_timing(
        &mut self,
        timing: Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    ) {
        self.timing = timing;
    }

    /// Installs [`crate::run_timing::RunTiming`] for this client before a timed prompt or multiturn run.
    #[must_use]
    pub fn attach_run_timing_for_session(
        &mut self,
    ) -> std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>> {
        crate::run_timing::attach_new_run_timing(&mut self.timing)
    }

    /// Returns existing run timing or installs a new wall clock when none is active.
    #[must_use]
    pub fn ensure_run_timing_for_session(
        &mut self,
    ) -> std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>> {
        if let Some(t) = self.timing.clone() {
            return t;
        }
        self.attach_run_timing_for_session()
    }

    pub(crate) fn set_timing_implement_display_name(&self, label: &'static str) {
        let Some(timing) = self.timing.as_ref() else {
            return;
        };
        timing
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .set_implement_display_name(label);
    }

    /// Verify API key env or `agent` / `cursor-agent` auth probes.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] when no credentials and probes fail.
    pub fn ensure_authenticated(&self) -> Result<(), AuthError> {
        if crate::acp::cursor_cli_auth_established() {
            return Ok(());
        }
        Err(AuthError(
            "Cursor agent is not authenticated for `agent acp`. Run `agent login` or set CURSOR_AGENT_API_KEY, CURSOR_API_KEY, or AGENT_API_KEY."
                .to_string(),
        ))
    }

    /// Returns true while a coder session is active (after [`Self::begin_coder_session`] succeeds, until [`Self::end_coder_session`]).
    #[must_use]
    pub const fn has_open_coder_session(&self) -> bool {
        self.coder_session.is_some()
    }

    /// Spawn the **coder** ACP session. Call once before [`Self::run_coder_prompt`].
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when spawn fails after retries, or when a coder session is already open.
    pub async fn begin_coder_session(&mut self, cwd: &Path) -> Result<(), AgentError> {
        if self.coder_session.is_some() {
            return Err(AgentError("coder ACP session is already open".to_string()));
        }
        let mut last_error = String::new();
        let mut attempts_used = 0_u32;
        let max_attempts = self.max_acp_retries;
        for attempt in 1..=max_attempts {
            attempts_used = attempt;
            match spawn_agent_acp_session(self, cwd).await {
                Ok(mut s) => {
                    crate::acp::acp_session_set_run_timing(&mut s, self.timing.clone());
                    self.coder_session = Some(s);
                    return Ok(());
                }
                Err(e) => {
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
        let retries = attempts_used.saturating_sub(1);
        let noun = retries_noun(retries);
        Err(AgentError(format!(
            "agent acp (coder session) failed to spawn after {retries} {noun}. Last error:\n{last_error}"
        )))
    }
}
