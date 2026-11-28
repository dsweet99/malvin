use std::path::Path;

use crate::agent_process::{AgentError, AgentFault, CoderPromptOptions};

use super::agent_backend::backend_lifecycle::backend_lifecycle;
use super::agent_backend::sdk_client::SdkClient;

pub(super) async fn teardown_sdk_session_after_transport_error(
    client: &mut SdkClient,
    err: &AgentError,
) {
    if !err.requires_coder_session_teardown() {
        return;
    }
    let forget_agent = backend_lifecycle(client.model.backend).forgets_resume_id_on_busy_teardown()
        && err.fault == AgentFault::CursorBusy;
    let _ = client.end_coder_session().await;
    if forget_agent {
        client.last_agent_id = None;
    }
}

pub(super) async fn force_fresh_agent_for_retry(client: &mut SdkClient) {
    let _ = client.end_coder_session().await;
    client.last_agent_id = None;
    client.header_lifecycle.require_redelivery();
}

pub(super) fn emit_prompt_stdout(
    client: &SdkClient,
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

pub(super) fn append_prompt_files(
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
