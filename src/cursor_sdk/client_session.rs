//! Session begin / end for [`super::CursorSdkClient`].

use std::path::Path;

use crate::acp::{backoff_after_agent_failure, retries_noun, AgentError, AuthError};

use super::auth::ensure_sdk_authenticated;
use super::client::CursorSdkClient;
use super::session::BridgeSession;

impl CursorSdkClient {
    /// # Errors
    ///
    /// Returns [`AuthError`] when no Cursor API key is configured.
    pub fn ensure_authenticated(&self) -> Result<(), AuthError> {
        ensure_sdk_authenticated()
    }

    /// # Errors
    ///
    /// Returns [`AgentError`] when spawn fails after retries.
    pub async fn begin_coder_session(&mut self, cwd: &Path) -> Result<(), AgentError> {
        if self.session.is_some() {
            return Err(AgentError("cursor SDK session is already open".into()));
        }
        let cwd = crate::acp::resolve_acp_session_cwd(cwd)?;
        let model = crate::model_id::provider_slug(&self.model);
        let mut last_error = String::new();
        let max_attempts = self.max_acp_retries;
        let mut attempts_used = 0_u32;
        for attempt in 1..=max_attempts {
            attempts_used = attempt;
            match BridgeSession::spawn(super::session::BridgeSpawnArgs {
                cwd: &cwd,
                model: &model,
                io: self.io,
                run_dir: self.prompts_log_run_dir.clone(),
                timing: self.timing.clone(),
            })
            .await
            {
                Ok(s) => {
                    self.session = Some(s);
                    self.session_cwd = Some(cwd);
                    crate::herdr::notify_reclaim();
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
        Err(AgentError(format!(
            "cursor-sdk-bridge failed to spawn after {retries} {}. Last error:\n{last_error}",
            retries_noun(retries)
        )))
    }

    /// # Errors
    ///
    /// Returns [`AgentError`] when shutdown fails.
    pub async fn end_coder_session(&mut self) -> Result<(), AgentError> {
        if let Some(s) = self.session.take() {
            s.shutdown().await?;
        }
        Ok(())
    }
}
