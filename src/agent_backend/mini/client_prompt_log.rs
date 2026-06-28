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
    if client.trace.plain_lines {
        return;
    }
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
    use crate::agent_backend::test_support::mini_loop_config;
    use crate::agent_backend::mini::{LlmBackend, MiniAgentClient, MockScript, MockStep};
    use malvin_mini::CompletionResponse;
    use std::sync::Mutex;

    fn test_client(verbose: bool) -> MiniAgentClient {
        MiniAgentClient::new_mock(
            mini_loop_config(4, 1),
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
                    reasoning: None,
                })],
                call_count: 0,
                on_response: None,
            })),
        )
    }

    #[tokio::test]
    async fn mini_do_prompt_log_skips_live_stdout_bracket() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut client = test_client(false);
        client.trace.plain_lines = true;
        let log = tmp.path().join("do.log");
        let log_path = log.clone();
        crate::output::set_stdout_log_path(Some(tmp.path().join("stdout.log")));
        write_prompt_log(PromptLogWrite {
            client: &client,
            prompt: "body",
            log_path: &log_path,
            who: "do",
            opts: &CoderPromptOptions {
                do_trace_split: Some(("header", "user")),
                ..Default::default()
            },
        })
        .expect("write");
        let stdout = std::fs::read_to_string(tmp.path().join("stdout.log")).unwrap_or_default();
        assert!(
            stdout.is_empty(),
            "plain do must not emit d|[do] bracket on stdout; got {stdout:?}"
        );
        crate::output::set_stdout_log_path(None);
    }

    #[tokio::test]
    async fn mini_write_prompt_log_includes_effective_constraints() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let mut client = test_client(true);
        client.prompts_log_run_dir = Some(tmp.path().to_path_buf());
        let log = tmp.path().join("kpop.log");
        write_prompt_log(PromptLogWrite {
            client: &client,
            prompt: "constraints block\n\nbody text",
            log_path: &log,
            who: "kpop",
            opts: &CoderPromptOptions::default(),
        })
        .expect("write");
        let text = std::fs::read_to_string(&log).expect("read");
        assert!(text.contains("constraints block"));
        assert!(text.contains("body text"));
        let run_prompts = std::fs::read_to_string(tmp.path().join("prompts.log")).expect("mirror");
        assert!(run_prompts.contains("constraints block"));
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
