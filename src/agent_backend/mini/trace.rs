//! Mini trace events to `trace.jsonl` and stdout.

use malvin_mini::ResponseUsage;

use crate::acp::AgentIoOptions;

pub struct MiniTraceSink {
    pub run_dir: Option<std::path::PathBuf>,
    pub io: AgentIoOptions,
}

impl MiniTraceSink {
    pub fn write_jsonl_event(&self, event_type: &str, payload: serde_json::Value) {
        let Some(run_dir) = self.run_dir.as_ref() else {
            return;
        };
        let path = run_dir.join("trace.jsonl");
        let line = serde_json::json!({
            "type": event_type,
            "ts": crate::time_format::timestamp_now_string(),
            "payload": payload,
        });
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(path) {
            use std::io::Write;
            let _ = writeln!(f, "{line}");
        }
    }

    pub fn mini_llm_request(&self, usage: Option<&ResponseUsage>) {
        let mut payload = serde_json::json!({});
        if let Some(u) = usage {
            payload["usage"] = serde_json::json!({
                "prompt_tokens": u.prompt_tokens,
                "completion_tokens": u.completion_tokens,
                "total_tokens": u.total_tokens,
                "cost": u.cost,
            });
        }
        self.write_jsonl_event("mini_llm_request", payload);
    }

    pub fn mini_bash_exec(&self, command: &str, exit_code: i32) {
        self.write_jsonl_event(
            "mini_bash_exec",
            serde_json::json!({ "command": command, "exit_code": exit_code }),
        );
        if !self.io.no_tee {
            let summary = format!("[bash] $ {command} (exit {exit_code})");
            crate::output::print_stdout_line("bash", &summary);
        }
    }

    pub fn mini_assistant(&self, text: &str) {
        self.write_jsonl_event(
            "mini_assistant",
            serde_json::json!({ "text_len": text.len() }),
        );
        if self.io.show_thoughts_on_stdout && !crate::output::stdout_suppressed() {
            crate::output::print_stdout_line("mini", text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mini_trace_writes_mini_llm_request_with_usage() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let sink = MiniTraceSink {
            run_dir: Some(tmp.path().to_path_buf()),
            io: crate::acp::AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        };
        sink.mini_llm_request(Some(&ResponseUsage {
            prompt_tokens: Some(1),
            completion_tokens: Some(2),
            total_tokens: Some(3),
            cost: Some(0.01),
        }));
        let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
        assert!(text.contains("mini_llm_request"));
        assert!(text.contains("cost"));
    }

    #[test]
    fn mini_trace_writes_mini_bash_exec() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let sink = MiniTraceSink {
            run_dir: Some(tmp.path().to_path_buf()),
            io: crate::acp::AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        };
        sink.mini_bash_exec("echo hi", 0);
        let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
        assert!(text.contains("mini_bash_exec"));
    }

    #[test]
    fn mini_trace_writes_mini_assistant() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let sink = MiniTraceSink {
            run_dir: Some(tmp.path().to_path_buf()),
            io: crate::acp::AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        };
        sink.mini_assistant("hello assistant");
        let text = std::fs::read_to_string(tmp.path().join("trace.jsonl")).expect("trace");
        assert!(text.contains("mini_assistant"));
        assert!(text.contains("text_len"));
    }
}
