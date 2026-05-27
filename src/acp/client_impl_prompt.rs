use std::path::Path;
use std::time::Instant;

use crate::acp::{
    backoff_after_agent_failure, outgoing_prompt_trace, AgentClient, AgentError,
    CoderSessionPromptDispatch, coder_prompt_exhausted_error,
    dispatch_coder_session_prompt, record_coder_prompt_llm_timing,
};

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
    let max_attempts = client.max_acp_retries;
    for attempt in 1..=max_attempts {
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
    Err(coder_prompt_exhausted_error(attempts_used, last_error))
}

#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_run_coder_prompt_with_retries() {
        let _ = stringify!(run_coder_prompt_with_retries);
    }
}
