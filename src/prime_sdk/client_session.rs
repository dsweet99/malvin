//! Session begin / end for [`super::PrimeSdkClient`] (fresh create only; no resume).

use std::path::{Path, PathBuf};

use crate::acp::{backoff_after_agent_failure, retries_noun, AgentError, AuthError};

use super::auth::ensure_prime_authenticated;
use super::client::PrimeSdkClient;
use super::session::{PrimeBridgeSession, PrimeBridgeSpawnArgs, SDK_BRIDGE_MAX_AGE};

impl PrimeSdkClient {
    /// # Errors
    ///
    /// Returns [`AuthError`] when no provider API key is configured (skipped for
    /// `prime:local/local/…`).
    pub fn ensure_authenticated(&self) -> Result<(), AuthError> {
        ensure_prime_authenticated(&self.model)
    }

    /// Open a coder session if needed. Restarts the Node bridge when aged out.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when spawn or shutdown fails after retries.
    pub async fn ensure_coder_session(&mut self, cwd: &Path) -> Result<(), AgentError> {
        if self.prime_sdk_bridge_needs_restart() {
            self.end_coder_session().await?;
        }
        if self.session.is_some() {
            return Ok(());
        }
        self.begin_coder_session(cwd).await
    }

    #[must_use]
    pub(crate) fn prime_sdk_bridge_needs_restart(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|s| s.started_at.elapsed() >= SDK_BRIDGE_MAX_AGE)
    }

    /// # Errors
    ///
    /// Returns [`AgentError`] when spawn fails after retries.
    pub async fn begin_coder_session(&mut self, cwd: &Path) -> Result<(), AgentError> {
        if self.session.is_some() {
            return Err(AgentError("prime SDK session is already open".into()));
        }
        let cwd = crate::acp::resolve_acp_session_cwd(cwd)?;
        let model = crate::model_id::provider_slug(&self.model);
        let mut last_error = String::new();
        let max_attempts = self.max_acp_retries;
        let mut attempts_used = 0_u32;
        for attempt in 1..=max_attempts {
            attempts_used = attempt;
            match PrimeBridgeSession::spawn(self.bridge_spawn_args(&cwd, &model)).await {
                Ok(s) => {
                    self.adopt_spawned_session(s, cwd);
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
            "prime-sdk-bridge failed to spawn after {retries} {}. Last error:\n{last_error}",
            retries_noun(retries)
        )))
    }

    fn bridge_spawn_args<'a>(&'a self, cwd: &'a Path, model: &'a str) -> PrimeBridgeSpawnArgs<'a> {
        PrimeBridgeSpawnArgs {
            cwd,
            model,
            io: self.io,
            run_dir: self.prompts_log_run_dir.clone(),
            timing: self.timing.clone(),
            allow_download: self.allow_download,
            prime_local: crate::model_id::uses_prime_local_backend(&self.model),
        }
    }

    fn adopt_spawned_session(&mut self, s: PrimeBridgeSession, cwd: PathBuf) {
        self.session = Some(s);
        self.session_cwd = Some(cwd);
        crate::herdr::notify_reclaim();
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
