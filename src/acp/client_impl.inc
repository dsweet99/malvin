/// Apply bounded-retry backoff after a failed attempt, or stop the retry loop.
/// Returns `Ok(true)` when the caller should `break` the attempt loop; `Err` on upgrade-plan short-circuit.
async fn backoff_after_agent_failure(last_error: &str, attempt: u32) -> Result<bool, AgentError> {
    match plan_agent_retry(last_error, attempt) {
        Err(e) => Err(e),
        Ok(AgentRetryOutcome::StopRetrying) => Ok(true),
        Ok(AgentRetryOutcome::Sleep(d)) => {
            sleep(d).await;
            Ok(false)
        }
    }
}

impl AgentClient {
    #[must_use]
    pub fn new(model: String, io: AgentIoOptions) -> Self {
        Self {
            model,
            io,
            style_prompt_path: PathBuf::from(".style").join("main.md"),
            coder_session: None,
            coder_style_on_next_prompt: false,
        }
    }

    /// Verify API key env or `agent` / `cursor-agent` auth probes.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError`] when no credentials and probes fail.
    pub fn ensure_authenticated(&self) -> Result<(), AuthError> {
        if has_api_key() {
            return Ok(());
        }
        if auth_probe(&["agent", "auth", "status"]) {
            return Ok(());
        }
        if auth_probe(&["cursor-agent", "auth", "status"]) {
            return Ok(());
        }
        if auth_probe(&["agent", "whoami"]) {
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
        let mut last_error = String::new();
        for attempt in 1..=MAX_AGENT_ATTEMPTS {
            match spawn_agent_acp_session(self, cwd).await {
                Ok(s) => {
                    self.coder_session = Some(s);
                    self.coder_style_on_next_prompt = true;
                    return Ok(());
                }
                Err(e) => {
                    last_error = e.0;
                    if backoff_after_agent_failure(&last_error, attempt).await? {
                        break;
                    }
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

        let mut last_error = String::new();
        let session = session.clone();

        for attempt in 1..=MAX_AGENT_ATTEMPTS {
            match session.prompt(&full_prompt, log_path).await {
                Ok(()) => {
                    maybe_tee_log(self, log_path);
                    return Ok(());
                }
                Err(e) => {
                    last_error = e;
                    if backoff_after_agent_failure(&last_error, attempt).await? {
                        break;
                    }
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
        let mut last_error = String::new();

        for attempt in 1..=MAX_AGENT_ATTEMPTS {
            match run_reviewer_pair_once(self, &pair).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = e.0;
                    if backoff_after_agent_failure(&last_error, attempt).await? {
                        break;
                    }
                }
            }
        }

        Err(AgentError(format!(
            "agent acp (reviewer review+kpop) failed after retries. Last error:\n{last_error}"
        )))
    }

    /// Standalone KPOP: one ACP session without `.style/main.md` injection; optional `learn.md` in the same session.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when spawn or a prompt fails after retries.
    pub async fn run_kpop_flow(
        &mut self,
        cwd: &Path,
        kpop_prompt: &str,
        kpop_log: &Path,
        learn: Option<(&str, &Path)>,
        p_creative: f64,
        mbc2_body: &str,
    ) -> Result<(), AgentError> {
        let mut last_error = String::new();

        for attempt in 1..=MAX_AGENT_ATTEMPTS {
            let once = KpopFlowOnceArgs {
                cwd,
                kpop_prompt,
                kpop_log,
                learn,
                p_creative,
                mbc2_body,
            };
            match run_kpop_flow_once(self, once).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = e.0;
                    if backoff_after_agent_failure(&last_error, attempt).await? {
                        break;
                    }
                }
            }
        }

        Err(AgentError(format!(
            "agent acp (kpop flow) failed after retries. Last error:\n{last_error}"
        )))
    }
}
