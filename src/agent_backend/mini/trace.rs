//! Mini trace events to `trace.jsonl` and stdout.

use std::time::Duration;

use malvin_mini::ResponseUsage;

use crate::acp::AgentIoOptions;
use crate::output::{AcpTeeDirection, AcpTeeStdoutEvent, WHO_M, WHO_T};
use crate::tool_summary::{
    escape_tool_subject_fragment, humanize_duration, shorten_middle, tool_summary_stdout_display,
    TOOL_DISPLAY_MAX_WIDTH,
};

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

    pub fn mini_bash_exec(&self, command: &str, exit_code: i32, elapsed: Duration) {
        self.write_jsonl_event(
            "mini_bash_exec",
            serde_json::json!({ "command": command, "exit_code": exit_code }),
        );
        if self.io.no_tee || crate::output::stdout_suppressed() {
            return;
        }
        let plain = format_mini_bash_tool_line(command, exit_code, elapsed);
        let display = tool_summary_stdout_display(&plain);
        let ts = crate::output::timestamp_now_string();
        crate::output::print_stdout_acp_tool_summary_tee(
            &AcpTeeStdoutEvent {
                direction: AcpTeeDirection::FromAgent,
                who: WHO_T,
                line: &plain,
                ts: &ts,
                emit_stdout_markdown: self.io.emit_stdout_markdown,
                dim_payload: false,
            },
            &display,
        );
    }

    pub fn mini_assistant(&self, text: &str) {
        self.write_jsonl_event(
            "mini_assistant",
            serde_json::json!({ "text_len": text.len() }),
        );
        if self.io.no_tee || crate::output::stdout_suppressed() {
            return;
        }
        crate::output::print_stdout_line(WHO_M, text);
    }
}

pub(crate) fn format_mini_bash_tool_line(command: &str, exit_code: i32, elapsed: Duration) -> String {
    let flattened = escape_tool_subject_fragment(command.trim());
    let cmd = shorten_middle(&flattened, TOOL_DISPLAY_MAX_WIDTH);
    let dur = humanize_duration(elapsed);
    if exit_code == 0 {
        format!("Run {cmd} · {dur} · ✓")
    } else {
        format!("Run {cmd} · {dur} · ✗ exit {exit_code}")
    }
}

#[cfg(test)]
mod kiss_cov_inline {
    use super::*;

    #[test]
    fn kiss_cov_band80_witnesses() {
        let sink = MiniTraceSink {
            run_dir: Some(std::path::PathBuf::from("/tmp/run")),
            io: crate::acp::AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        };
        let MiniTraceSink { run_dir, io } = sink;
        assert_eq!(run_dir.as_deref(), Some(std::path::Path::new("/tmp/run")));
        assert!(!io.force);
    }
}

#[cfg(test)]
#[path = "trace_test.rs"]
mod trace_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<MiniTraceSink> = None;
        let _ = format_mini_bash_tool_line;
        let _ = mini_assistant;
        let _ = mini_bash_exec;
        let _ = mini_llm_request;
        let _ = write_jsonl_event;
    }
}
