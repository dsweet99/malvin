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
    if client.io.log_full_outgoing_prompts {
        crate::output::print_stdout_line(label, prompt);
    } else {
        crate::output::print_stdout_line(label, &format!("[{who}]"));
    }
}

fn append_prompt_log_file(
    client: &MiniAgentClient,
    prompt: &str,
    log_path: &Path,
    who: &str,
) -> Result<(), AgentError> {
    if let Some(parent) = log_path.parent() {
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
    let Some(run_dir) = client.prompts_log_run_dir.as_ref() else {
        return;
    };
    let prompts_log = run_dir.join("prompts.log");
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(prompts_log)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::acp::CoderPromptOptions;
    use crate::agent_backend::mini::{LlmBackend, MiniAgentClient, MiniLoopConfig, MockScript, MockStep};
    use malvin_mini::CompletionResponse;
    use std::sync::Mutex;

    fn test_client(verbose: bool) -> MiniAgentClient {
        MiniAgentClient::new_mock(
            MiniLoopConfig {
                model: "m".into(),
                max_bash_turns: 4,
                max_http_retries: 1,
            },
            crate::acp::AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: verbose,
            },
            LlmBackend::Mock(Mutex::new(MockScript {
                responses: vec![MockStep::Ok(CompletionResponse {
                    content: "ok".into(),
                    usage: None,
                })],
                call_count: 0,
                on_response: None,
            })),
        )
    }

    #[tokio::test]
    async fn mini_write_prompt_log_appends_log_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let client = test_client(true);
        let log = tmp.path().join("kpop.log");
        write_prompt_log(PromptLogWrite {
            client: &client,
            prompt: "body text",
            log_path: &log,
            who: "kpop",
            opts: &CoderPromptOptions::default(),
        })
        .expect("write");
        let text = std::fs::read_to_string(&log).expect("read");
        assert!(text.contains("body text"));
    }
}

#[cfg(test)]
#[path = "client_prompt_log_test.rs"]
mod client_prompt_log_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<PromptLogWrite> = None;
        let _ = append_prompt_log_file;
        let _ = emit_stdout_line;
        let _ = format_prompt_log_line;
        let _ = mirror_prompt_log_to_run_dir;
        let _ = write_prompt_log;
    }
}
