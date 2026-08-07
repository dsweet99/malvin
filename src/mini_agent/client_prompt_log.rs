//! Prompt log writes for [`super::client::MiniAgentClient`].

use std::path::Path;

use crate::acp::{AgentError, CoderPromptOptions};

use super::client::MiniAgentClient;

pub struct PromptLogWrite<'a> {
    pub client: &'a MiniAgentClient,
    pub prompt: &'a str,
    pub log_path: &'a Path,
    pub who: &'a str,
    pub opts: &'a CoderPromptOptions<'a>,
}

pub fn write_prompt_log(ctx: PromptLogWrite<'_>) -> Result<(), AgentError> {
    let PromptLogWrite {
        client,
        prompt,
        log_path,
        who,
        opts,
    } = ctx;
    let label = opts.stdout_bracket_label.unwrap_or(who);
    emit_stdout_line(client, label, prompt, who);
    append_prompt_log_file(client, prompt, log_path, who)?;
    Ok(())
}

fn emit_stdout_line(client: &MiniAgentClient, label: &str, prompt: &str, who: &str) {
    // Match cursor:/prime: client_prompt: skip stdout.log brackets when tee is off (`-b`).
    if client.trace.plain_lines || client.io.raw_output || client.io.no_tee {
        return;
    }
    crate::output::print_outgoing_prompt_log(who, label);
    if client.io.log_full_outgoing_prompts {
        crate::output::append_outgoing_prompt_log_lines(prompt);
    }
}

fn append_prompt_log_file(
    client: &MiniAgentClient,
    prompt: &str,
    log_path: &Path,
    who: &str,
) -> Result<(), AgentError> {
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let line = format_prompt_log_line(client, prompt, who);
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()))
        .map_err(|e| AgentError(format!("prompt log write failed: {e}")))?;
    mirror_prompt_log_to_run_dir(client, &line);
    Ok(())
}

fn format_prompt_log_line(client: &MiniAgentClient, prompt: &str, who: &str) -> String {
    let mut line = format!("{} {who}\n", crate::time_format::timestamp_now_string());
    if client.io.log_full_outgoing_prompts {
        line.push_str(prompt);
        if !prompt.ends_with('\n') {
            line.push('\n');
        }
    }
    line
}

fn mirror_prompt_log_to_run_dir(client: &MiniAgentClient, line: &str) {
    let Some(run_dir) = client.trace_run_dir.as_ref() else {
        return;
    };
    let prompts_log = run_dir.join("prompts.log");
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(prompts_log)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
}
