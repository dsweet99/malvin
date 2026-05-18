impl AgentClient {
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
            .ok_or_else(|| AgentError("begin_coder_session was not called".to_string()))?
            .clone();

        let (full_prompt, repo_style) = coder_prompt_body_with_optional_repo_style(
            prompt,
            CoderPromptRepoStyleOpts {
                style_on_first_turn: self.coder_style_on_next_prompt,
                skip_repo_style,
            },
            &self.style_prompt_path,
        );
        crate::prompts::enforce_no_unresolved_braces(&full_prompt).map_err(|e| AgentError(e.0))?;

        let style_for_do_trace = if do_trace_split.is_some() {
            repo_style.as_deref()
        } else {
            None
        };

        self.coder_style_on_next_prompt = false;

        let dispatch = CoderSessionPromptDispatch {
            session: &session,
            full_prompt: &full_prompt,
            log_path,
            who,
            do_trace_split,
            style_for_do_trace,
            stdout_bracket_label,
        };
        run_coder_prompt_with_retries(self, dispatch, llm_phase).await
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
}

async fn run_coder_prompt_with_retries(
    client: &mut AgentClient,
    dispatch: CoderSessionPromptDispatch<'_>,
    llm_phase: Option<crate::run_timing::TimingPhase>,
) -> Result<(), AgentError> {
    let mut last_error = String::new();
    let mut attempts_used = 0_u32;
    for attempt in 1..=MAX_AGENT_ATTEMPTS {
        attempts_used = attempt;
        let t0 = Instant::now();
        let prompt_res = dispatch_coder_session_prompt(&dispatch).await;
        match prompt_res {
            Ok(()) => {
                record_coder_prompt_llm_timing(client.timing.as_ref(), llm_phase, t0.elapsed());
                return Ok(());
            }
            Err(e) => {
                record_coder_prompt_llm_timing(client.timing.as_ref(), llm_phase, t0.elapsed());
                last_error = e;
                if backoff_after_agent_failure(client.timing.as_ref(), &last_error, attempt)
                    .await?
                {
                    break;
                }
            }
        }
    }
    Err(coder_prompt_exhausted_error(attempts_used, last_error))
}
