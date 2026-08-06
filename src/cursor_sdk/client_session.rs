//! Session begin / end for [`super::CursorSdkClient`].

use std::path::{Path, PathBuf};

use crate::acp::{backoff_after_agent_failure, retries_noun, AgentError, AuthError};

use super::auth::ensure_sdk_authenticated;
use super::client::CursorSdkClient;
use super::session::{BridgeSession, BridgeSpawnArgs, SDK_BRIDGE_MAX_AGE};

impl CursorSdkClient {
    /// # Errors
    ///
    /// Returns [`AuthError`] when no Cursor API key is configured.
    pub fn ensure_authenticated(&self) -> Result<(), AuthError> {
        ensure_sdk_authenticated()
    }

    /// Open a coder session if needed. Restarts the Node bridge when it is at
    /// least [`SDK_BRIDGE_MAX_AGE`] old so Cursor-side idle timeouts do not break
    /// later agent turns.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when spawn or shutdown fails after retries.
    pub async fn ensure_coder_session(&mut self, cwd: &Path) -> Result<(), AgentError> {
        if self.sdk_bridge_needs_restart() {
            self.end_coder_session().await?;
        }
        if self.session.is_some() {
            return Ok(());
        }
        self.begin_coder_session(cwd).await
    }

    #[must_use]
    pub(crate) fn sdk_bridge_needs_restart(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|s| s.started_at.elapsed() >= SDK_BRIDGE_MAX_AGE)
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
            match BridgeSession::spawn(self.bridge_spawn_args(&cwd, &model)).await {
                Ok(s) => {
                    self.adopt_spawned_session(s, cwd);
                    return Ok(());
                }
                Err(e) => {
                    last_error = self.note_spawn_failure(e);
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

    fn bridge_spawn_args<'a>(&'a self, cwd: &'a Path, model: &'a str) -> BridgeSpawnArgs<'a> {
        BridgeSpawnArgs {
            cwd,
            model,
            io: self.io,
            run_dir: self.prompts_log_run_dir.clone(),
            timing: self.timing.clone(),
            resume_agent_id: self.last_agent_id.clone(),
        }
    }

    fn adopt_spawned_session(&mut self, s: BridgeSession, cwd: PathBuf) {
        self.remember_agent_id_from(&s);
        self.session = Some(s);
        self.session_cwd = Some(cwd);
        crate::herdr::notify_reclaim();
    }

    fn note_spawn_failure(&mut self, err: AgentError) -> String {
        let mut last_error = err.0;
        // Resume can fail after long gaps; fall back to create on next try.
        if self.last_agent_id.take().is_some() {
            last_error = format!("{last_error} (resume failed; will create)");
        }
        last_error
    }

    fn remember_agent_id_from(&mut self, session: &BridgeSession) {
        let id = session
            .agent_id
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if let Some(id) = id {
            self.last_agent_id = Some(id);
        }
    }

    /// # Errors
    ///
    /// Returns [`AgentError`] when shutdown fails.
    pub async fn end_coder_session(&mut self) -> Result<(), AgentError> {
        if let Some(s) = self.session.take() {
            self.remember_agent_id_from(&s);
            s.shutdown().await?;
        }
        Ok(())
    }
}
