//! Prompt turns for [`super::PrimeSdkClient`].

use std::path::Path;

use crate::acp::{
    agent_error_requires_coder_session_teardown, backoff_after_agent_failure, retries_noun,
    AgentError, CoderPromptOptions,
};

use super::client::PrimeSdkClient;

impl PrimeSdkClient {
    /// # Errors
    ///
    /// Returns [`AgentError`] when there is no session or the prompt fails.
    pub async fn run_coder_prompt(
        &mut self,
        prompt: &str,
        log_path: &Path,
        who: &str,
        opts: CoderPromptOptions<'_>,
    ) -> Result<(), AgentError> {
        if self.session.is_none() && self.session_cwd.is_none() {
            return Err(AgentError("begin_coder_session was not called".into()));
        }
        prime_emit_prompt_stdout(self, prompt, who, &opts);
        prime_append_prompt_files(self, prompt, log_path, who)?;
        let single = opts.single_attempt;
        let max_attempts = if single { 1 } else { self.max_acp_retries };
        let mut last_error = String::new();
        for attempt in 1..=max_attempts {
            let phase = opts.llm_phase;
            match prime_run_one(self, prompt, phase).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = e.0;
                    prime_teardown_sdk_session_after_transport_error(self, &last_error).await;
                    if single {
                        break;
                    }
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
        let retries = max_attempts.saturating_sub(1);
        Err(AgentError(format!(
            "prime SDK prompt failed after {retries} {}. Last error:\n{last_error}",
            retries_noun(retries)
        )))
    }
}

async fn prime_teardown_sdk_session_after_transport_error(client: &mut PrimeSdkClient, err: &str) {
    if agent_error_requires_coder_session_teardown(err) {
        let _ = client.end_coder_session().await;
    }
}

async fn prime_run_one(
    client: &mut PrimeSdkClient,
    prompt: &str,
    phase: Option<crate::run_timing::TimingPhase>,
) -> Result<(), AgentError> {
    prime_ensure_open_session(client).await?;
    let session = client
        .session
        .as_ref()
        .ok_or_else(|| AgentError("begin_coder_session was not called".into()))?;
    let started = std::time::Instant::now();
    let result = session.send_prompt(prompt).await;
    if let Some(p) = phase {
        crate::run_timing::record_llm(client.timing.as_ref(), p, started.elapsed());
    }
    result
}

async fn prime_ensure_open_session(client: &mut PrimeSdkClient) -> Result<(), AgentError> {
    if client.session.is_some() {
        return Ok(());
    }
    let cwd = client
        .session_cwd
        .clone()
        .ok_or_else(|| AgentError("begin_coder_session was not called".into()))?;
    client.begin_coder_session(&cwd).await
}

fn prime_emit_prompt_stdout(
    client: &PrimeSdkClient,
    prompt: &str,
    who: &str,
    opts: &CoderPromptOptions<'_>,
) {
    if client.io.raw_output || client.io.no_tee {
        return;
    }
    let label = opts.stdout_bracket_label.unwrap_or(who);
    crate::output::print_outgoing_prompt_log(who, label);
    if client.io.log_full_outgoing_prompts {
        crate::output::append_outgoing_prompt_log_lines(prompt);
    }
}

fn prime_append_prompt_files(
    client: &PrimeSdkClient,
    prompt: &str,
    log_path: &Path,
    who: &str,
) -> Result<(), AgentError> {
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let line = prime_format_prompt_line(client, prompt, who);
    prime_append_prompt_log_bytes(log_path, line.as_bytes())?;
    if let Some(run_dir) = client.prompts_log_run_dir.as_ref() {
        let _ = prime_append_prompt_log_bytes(&run_dir.join("prompts.log"), line.as_bytes());
    }
    Ok(())
}

fn prime_append_prompt_log_bytes(path: &Path, bytes: &[u8]) -> Result<(), AgentError> {
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, bytes))
        .map_err(|e| AgentError(format!("prompt log write failed: {e}")))
}

fn prime_format_prompt_line(client: &PrimeSdkClient, prompt: &str, who: &str) -> String {
    let mut line = format!("{} {who}\n", crate::time_format::timestamp_now_string());
    if client.io.log_full_outgoing_prompts {
        line.push_str(prompt);
        if !prompt.ends_with('\n') {
            line.push('\n');
        }
    }
    line
}
