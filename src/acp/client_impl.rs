/// Repo-relative path (under the workflow working directory) for optional injected style text.
///
/// [`AgentClient::new`](crate::AgentClient::new) sets [`AgentClient`](crate::AgentClient)’s
/// `style_prompt_path` to this string by default. Documented in repository `grounding.md`.
pub const DEFAULT_REPO_STYLE_PROMPT_REL: &str = "coder_style.md";

/// Read optional repo-local style text (trimmed) with the same rules as coder prompt composition.
///
/// Returns `None` when the file is missing, unreadable, or whitespace-only after trim.
pub(crate) fn read_coder_repo_style_text(style_prompt_path: &Path) -> Option<String> {
    std::fs::read_to_string(style_prompt_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|t| !t.is_empty())
}

/// Prefix non-empty trimmed style before `prompt`, matching [`coder_prompt_body_with_optional_repo_style`].
pub(crate) fn prepend_coder_repo_style_to_prompt(prompt: &str, style_trimmed: Option<&str>) -> String {
    style_trimmed
        .filter(|t| !t.is_empty())
        .map_or_else(|| prompt.to_string(), |t| format!("{t}\n\n{prompt}"))
}

/// Build full prompt text and optional repo style read in one pass (at most one `read_to_string` on the style path).
pub(crate) fn coder_prompt_body_with_optional_repo_style(
    prompt: &str,
    style_on_first_turn: bool,
    skip_repo_style: bool,
    style_prompt_path: &Path,
) -> (String, Option<String>) {
    let repo_style = if style_on_first_turn && !skip_repo_style {
        read_coder_repo_style_text(style_prompt_path)
    } else {
        None
    };
    let full_prompt = prepend_coder_repo_style_to_prompt(prompt, repo_style.as_deref());
    (full_prompt, repo_style)
}

/// Apply bounded-retry backoff after a failed attempt, or stop the retry loop.
/// Returns `Ok(true)` when the caller should `break` the attempt loop; `Err` on upgrade-plan short-circuit.
async fn backoff_after_agent_failure(
    timing: Option<&std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_error: &str,
    attempt: u32,
) -> Result<bool, AgentError> {
    match plan_agent_retry(last_error, attempt) {
        Err(e) => Err(e),
        Ok(AgentRetryOutcome::StopRetrying) => Ok(true),
        Ok(AgentRetryOutcome::Sleep(d)) => {
            crate::run_timing::record_backoff(timing, d);
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
            style_prompt_path: PathBuf::from(DEFAULT_REPO_STYLE_PROMPT_REL),
            coder_session: None,
            coder_style_on_next_prompt: false,
            timing: None,
        }
    }

    /// When set (orchestrator, standalone KPOP), LLM waits and retry backoff are recorded.
    pub fn set_run_timing(
        &mut self,
        timing: Option<std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    ) {
        self.timing = timing;
    }

    /// Installs [`crate::run_timing::RunTiming`] for this session (`malvin code`, `malvin kpop`, and `malvin do`).
    #[must_use]
    pub fn attach_run_timing_for_session(
        &mut self,
    ) -> std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>> {
        crate::run_timing::attach_new_run_timing(&mut self.timing)
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
        let mut attempts_used = 0_u32;
        for attempt in 1..=MAX_AGENT_ATTEMPTS {
            attempts_used = attempt;
            match spawn_agent_acp_session(self, cwd).await {
                Ok(s) => {
                    self.coder_session = Some(s);
                    self.coder_style_on_next_prompt = true;
                    return Ok(());
                }
                Err(e) => {
                    last_error = e.0;
                    if backoff_after_agent_failure(self.timing.as_ref(), &last_error, attempt).await? {
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

    /// Run one prompt on the open coder session (implement, concerns, or learn).
    ///
    /// When `opts.skip_repo_style` is false, the first prompt after [`Self::begin_coder_session`] may prepend
    /// injected repo style when present. Set `skip_repo_style` for default raw `malvin do` so only `prompt` is sent.
    ///
    /// `who` names the outbound/inbound **trace stem** when `opts.do_trace_split` is `None` (for example
    /// `implement` for `implement.md`). `opts.stdout_bracket_label`
    /// overrides the stdout `[label...]` line and is usually the template filename (for example
    /// `implement.md`); pass `None` to default the bracket label to `who`. When `do_trace_split` is `Some`,
    /// stems come from the split trace (`>style` / `>header` / `>prompt`) and `who` / `stdout_bracket_label`
    /// are not used for the split path (the `malvin do` subcommand passes `"raw"` or `"header"` as `who` for trace only).
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
            skip_repo_style,
            do_trace_split,
            stdout_bracket_label,
        } = opts;
        let session = self
            .coder_session
            .as_ref()
            .ok_or_else(|| AgentError("begin_coder_session was not called".to_string()))?;

        let (full_prompt, repo_style) = coder_prompt_body_with_optional_repo_style(
            prompt,
            self.coder_style_on_next_prompt,
            skip_repo_style,
            &self.style_prompt_path,
        );

        let style_for_do_trace = if do_trace_split.is_some() {
            repo_style.as_deref()
        } else {
            None
        };

        self.coder_style_on_next_prompt = false;

        let mut last_error = String::new();
        let session = session.clone();

        let mut attempts_used = 0_u32;
        for attempt in 1..=MAX_AGENT_ATTEMPTS {
            attempts_used = attempt;
            let t0 = Instant::now();
            let prompt_res = match do_trace_split {
                None => session
                    .prompt(&full_prompt, log_path, who, stdout_bracket_label)
                    .await,
                Some((header, user)) => {
                    session
                        .prompt_do_trace_split(
                            &full_prompt,
                            log_path,
                            DoPromptTraceSplit {
                                style_text: style_for_do_trace,
                                header,
                                user,
                            },
                        )
                        .await
                }
            };
            match prompt_res {
                Ok(()) => {
                    if let Some(ph) = llm_phase {
                        crate::run_timing::record_llm(self.timing.as_ref(), ph, t0.elapsed());
                    }
                    return Ok(());
                }
                Err(e) => {
                    if let Some(ph) = llm_phase {
                        crate::run_timing::record_llm(self.timing.as_ref(), ph, t0.elapsed());
                    }
                    last_error = e;
                    if backoff_after_agent_failure(self.timing.as_ref(), &last_error, attempt).await? {
                        break;
                    }
                }
            }
        }

        let retries = attempts_used.saturating_sub(1);
        let noun = retries_noun(retries);
        Err(AgentError(format!(
            "agent acp (coder prompt) failed after {retries} {noun}. Last error:\n{last_error}"
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

    /// One **reviewer** session: spawns ACP, sends the review prompt, then shuts down.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when spawn or the prompt fails after retries.
    pub async fn run_reviewer_review(
        &mut self,
        pair: ReviewerPromptPair<'_>,
        pair_id: crate::run_timing::ReviewPairId,
    ) -> Result<(), AgentError> {
        let mut last_error = String::new();

        let mut attempts_used = 0_u32;
        for attempt in 1..=MAX_AGENT_ATTEMPTS {
            attempts_used = attempt;
            match run_reviewer_pair_once(self, &pair, pair_id).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = e.0;
                    if backoff_after_agent_failure(self.timing.as_ref(), &last_error, attempt).await? {
                        break;
                    }
                }
            }
        }

        let retries = attempts_used.saturating_sub(1);
        let noun = retries_noun(retries);
        Err(AgentError(format!(
            "agent acp (reviewer) failed after {retries} {noun}. Last error:\n{last_error}"
        )))
    }

    /// Standalone KPOP: one ACP session without injected repo style; optional `learn.md` in the same session.
    ///
    /// # Errors
    ///
    /// Returns [`AgentError`] when spawn or a prompt fails after retries.
    pub async fn run_kpop_flow(
        &mut self,
        flow: &KpopFlowOnceArgs<'_>,
    ) -> Result<(), AgentError> {
        let mut last_error = String::new();

        let mut attempts_used = 0_u32;
        for attempt in 1..=MAX_AGENT_ATTEMPTS {
            attempts_used = attempt;
            match run_kpop_flow_once(self, flow).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = e.0;
                    if backoff_after_agent_failure(self.timing.as_ref(), &last_error, attempt).await? {
                        break;
                    }
                }
            }
        }

        let retries = attempts_used.saturating_sub(1);
        let noun = retries_noun(retries);
        Err(AgentError(format!(
            "agent acp (kpop flow) failed after {retries} {noun}. Last error:\n{last_error}"
        )))
    }
}
