use std::path::Path;

use crate::acp::{
    AgentError, AgentFault, CoderPromptOptions, agent_string_is_cursor_agent_busy,
    backoff_after_agent_failure, retries_noun,
};
use crate::model_id::ModelBackend;

use super::sdk_client::SdkClient;

impl SdkClient {
    pub async fn run_coder_prompt(
        &mut self,
        prompt: &str,
        log_path: &Path,
        who: &str,
        opts: CoderPromptOptions<'_>,
    ) -> Result<(), AgentError> {
        if self.coder.is_none() {
            return Err(AgentError("begin_coder_session was not called".into()));
        }
        emit_prompt_stdout(self, prompt, who, &opts);
        append_prompt_files(self, prompt, log_path, who)?;
        let single = opts.single_attempt;
        let max_attempts = if single { 1 } else { self.max_acp_retries };
        let mut last_error = String::new();
        for attempt in 1..=max_attempts {
            let phase = opts.llm_phase;
            match run_one(self, prompt, phase).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    teardown_sdk_session_after_transport_error(self, &e).await;
                    last_error = e.message;
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
            "{} SDK prompt failed after {retries} {}. Last error:\n{last_error}",
            self.model.backend.label(),
            retries_noun(retries)
        )))
    }
}

async fn teardown_sdk_session_after_transport_error(client: &mut SdkClient, err: &AgentError) {
    if !err.requires_coder_session_teardown() {
        return;
    }
    let forget_agent = matches!(client.model.backend, ModelBackend::Cursor)
        && (err.fault == AgentFault::CursorBusy
            || agent_string_is_cursor_agent_busy(&err.message));
    let _ = client.end_coder_session().await;
    if forget_agent {
        client.last_agent_id = None;
    }
}

async fn run_one(
    client: &mut SdkClient,
    prompt: &str,
    phase: Option<crate::run_timing::TimingPhase>,
) -> Result<(), AgentError> {
    ensure_open_session(client).await?;
    let session = super::sdk_client::live_session(client)
        .ok_or_else(|| AgentError("begin_coder_session was not called".into()))?;
    let started = std::time::Instant::now();
    let result = session.send_prompt(prompt).await;
    if let Some(p) = phase {
        crate::run_timing::record_llm(client.timing.as_ref(), p, started.elapsed());
    }
    result
}

async fn ensure_open_session(client: &mut SdkClient) -> Result<(), AgentError> {
    if client.has_open_coder_session() {
        return Ok(());
    }
    let cwd = super::sdk_client::begun_cwd(client)
        .cloned()
        .ok_or_else(|| AgentError("begin_coder_session was not called".into()))?;
    client.begin_coder_session(&cwd).await
}

fn emit_prompt_stdout(client: &SdkClient, prompt: &str, who: &str, opts: &CoderPromptOptions<'_>) {
    if client.io.raw_output || client.io.no_tee {
        return;
    }
    let label = opts.stdout_bracket_label.unwrap_or(who);
    crate::output::print_outgoing_prompt_log(who, label);
    if client.io.log_full_outgoing_prompts {
        crate::output::append_outgoing_prompt_log_lines(prompt);
    }
}

fn append_prompt_files(
    client: &SdkClient,
    prompt: &str,
    log_path: &Path,
    who: &str,
) -> Result<(), AgentError> {
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let line = format_prompt_line(client, prompt, who);
    append_prompt_log_bytes(log_path, line.as_bytes())?;
    if let Some(run_dir) = client.prompts_log_run_dir.as_ref() {
        let _ = append_prompt_log_bytes(&run_dir.join("prompts.log"), line.as_bytes());
    }
    Ok(())
}

fn append_prompt_log_bytes(path: &Path, bytes: &[u8]) -> Result<(), AgentError> {
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, bytes))
        .map_err(|e| AgentError(format!("prompt log write failed: {e}")))
}

fn format_prompt_line(client: &SdkClient, prompt: &str, who: &str) -> String {
    let mut line = format!("{} {who}\n", crate::time_format::timestamp_now_string());
    if client.io.log_full_outgoing_prompts {
        line.push_str(prompt);
        if !prompt.ends_with('\n') {
            line.push('\n');
        }
    }
    line
}
