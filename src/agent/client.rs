use std::path::{Path, PathBuf};
use std::time::Duration;

use tokio::time::sleep;

use crate::acp::AcpSession;

use super::ops;
use super::pair::ReviewerPromptPair;
use super::{AgentError, AgentIoOptions, AuthError};

/// ACP-backed agent with grounding-aligned session lifetimes.
///
/// Uses one long-lived **coder** session (implement → concerns → learn) and one **reviewer**
/// session per attempt (review + kpop), torn down after each pair.
pub struct AgentClient {
    pub model: String,
    pub io: AgentIoOptions,
    retries: u32,
    pub(super) style_prompt_path: PathBuf,
    coder_session: Option<AcpSession>,
    /// When true, the next [`Self::run_coder_prompt`] prepends `.style/main.md` (first turn only).
    coder_style_on_next_prompt: bool,
}

impl AgentClient {
    #[must_use]
    pub fn new(model: String, io: AgentIoOptions) -> Self {
        Self {
            model,
            io,
            retries: 3,
            style_prompt_path: PathBuf::from(".style").join("main.md"),
            coder_session: None,
            coder_style_on_next_prompt: false,
        }
    }

    /// Verify API key env or `agent` / `cursor-agent` auth probes (same intent as Python malvin).
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] when no credentials and probes fail.
    pub fn ensure_authenticated(&self) -> Result<(), AuthError> {
        if ops::has_api_key() {
            return Ok(());
        }
        if ops::auth_probe(&["agent", "auth", "status"]) {
            return Ok(());
        }
        if ops::auth_probe(&["cursor-agent", "auth", "status"]) {
            return Ok(());
        }
        if ops::auth_probe(&["agent", "whoami"]) {
            return Ok(());
        }
        Err(AuthError(
            "Cursor agent is not authenticated for `agent acp`. Run `agent login` or set CURSOR_AGENT_API_KEY, CURSOR_API_KEY, or AGENT_API_KEY."
                .to_string(),
        ))
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
        let mut delay = 1.0f64;
        let mut last_error = String::new();
        let mut attempt = 0u32;
        while attempt <= self.retries {
            attempt += 1;
            match ops::spawn_acp_session(self, cwd).await {
                Ok(s) => {
                    self.coder_session = Some(s);
                    self.coder_style_on_next_prompt = true;
                    return Ok(());
                }
                Err(e) => {
                    last_error = e.0;
                    if attempt > self.retries {
                        break;
                    }
                    sleep(Duration::from_secs_f64(delay)).await;
                    delay = (delay * 2.0).min(8.0);
                }
            }
        }
        Err(AgentError(format!(
            "agent acp (coder session) failed to spawn after retries. Last error:\n{last_error}"
        )))
    }

    /// Run one prompt on the open coder session (implement, concerns, or learn).
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when there is no session or the prompt fails after retries.
    pub async fn run_coder_prompt(&mut self, prompt: &str, log_path: &Path) -> Result<(), AgentError> {
        let session = self
            .coder_session
            .as_ref()
            .ok_or_else(|| AgentError("begin_coder_session was not called".to_string()))?;

        let mut full_prompt = prompt.to_string();
        if self.coder_style_on_next_prompt
            && let Ok(style_text) = std::fs::read_to_string(&self.style_prompt_path)
        {
            let t = style_text.trim();
            if !t.is_empty() {
                full_prompt = format!("{t}\n\n{full_prompt}");
            }
        }
        self.coder_style_on_next_prompt = false;

        let mut attempt = 0u32;
        let mut delay = 1.0f64;
        let mut last_error = String::new();
        let session = session.clone();

        while attempt <= self.retries {
            attempt += 1;
            match session.prompt(&full_prompt, log_path).await {
                Ok(()) => {
                    ops::maybe_tee_log(self, log_path).await;
                    return Ok(());
                }
                Err(e) => {
                    last_error = e;
                    if attempt > self.retries {
                        break;
                    }
                    sleep(Duration::from_secs_f64(delay)).await;
                    delay = (delay * 2.0).min(8.0);
                }
            }
        }

        Err(AgentError(format!(
            "agent acp (coder prompt) failed after retries. Last error:\n{last_error}"
        )))
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

    /// One **reviewer** session: `review` then `kpop` (same ACP session, two `session/prompt` calls).
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when spawn or either prompt fails after retries.
    pub async fn run_reviewer_review_and_kpop(
        &mut self,
        pair: ReviewerPromptPair<'_>,
    ) -> Result<(), AgentError> {
        let mut attempt = 0u32;
        let mut delay = 1.0f64;
        let mut last_error = String::new();

        while attempt <= self.retries {
            attempt += 1;
            match ops::run_reviewer_pair_once(self, &pair).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = e.0;
                    if attempt > self.retries {
                        break;
                    }
                    sleep(Duration::from_secs_f64(delay)).await;
                    delay = (delay * 2.0).min(8.0);
                }
            }
        }

        Err(AgentError(format!(
            "agent acp (reviewer review+kpop) failed after retries. Last error:\n{last_error}"
        )))
    }
}
