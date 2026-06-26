//! Mini trace events to `trace.jsonl` and stdout.

use std::time::Duration;

use malvin_mini::ResponseUsage;

use super::acp_trace_shim::{
    append_out_raw, emit_agent_message_chunk, emit_bash_tool_call, emit_llm_usage,
    trace_for_run_dir,
};
use crate::acp::AgentIoOptions;
use crate::output::{AcpTeeDirection, AcpTeeStdoutEvent, WHO_M, WHO_T};
use crate::tool_summary::{
    bash_kind_wire_name, classify_bash_command, format_classified_tool_line,
    tool_summary_stdout_display,
};

pub struct MiniTraceSink {
    pub run_dir: Option<std::path::PathBuf>,
    pub io: AgentIoOptions,
}

impl MiniTraceSink {
    #[must_use]
    pub const fn new(run_dir: Option<std::path::PathBuf>, io: AgentIoOptions) -> Self {
        Self { run_dir, io }
    }

    fn acp_trace(&self) -> Option<crate::acp::AcpJsonlTrace> {
        self.run_dir
            .as_ref()
            .map(|dir| trace_for_run_dir(dir.as_path()))
    }

    pub fn append_prompts_log(&self, who: &str, body: &str) {
        let Some(run_dir) = self.run_dir.as_ref() else {
            return;
        };
        let mut line = format!("{} {who}\n", crate::time_format::timestamp_now_string());
        line.push_str(body);
        if !body.ends_with('\n') {
            line.push('\n');
        }
        let path = run_dir.join("prompts.log");
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
    }

    pub fn log_outgoing_prompt(&self, text: &str) {
        if let Some(trace) = self.acp_trace() {
            append_out_raw(&trace, text);
        }
    }

    pub fn log_nudge(&self, text: &str) {
        self.append_prompts_log("nudge", text);
        if let Some(trace) = self.acp_trace() {
            append_out_raw(&trace, text);
        }
    }

    pub fn mini_llm_request(&self, usage: Option<&ResponseUsage>) {
        if let (Some(trace), Some(u)) = (self.acp_trace(), usage) {
            emit_llm_usage(&trace, u);
        }
    }

    pub fn mini_bash_exec(&self, command: &str, exit_code: i32, elapsed: Duration) {
        let kind = classify_bash_command(command);
        if let Some(trace) = self.acp_trace() {
            emit_bash_tool_call(&trace, bash_kind_wire_name(kind), command, exit_code);
        }
        if self.io.no_tee || crate::output::stdout_suppressed() {
            return;
        }
        let plain = format_classified_tool_line(kind, command, exit_code, elapsed);
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

    pub fn stream_assistant_chunks(&self, text: &str) {
        if let Some(trace) = self.acp_trace() {
            for chunk in assistant_chunks(text) {
                emit_agent_message_chunk(&trace, chunk);
            }
        }
        if self.io.no_tee || crate::output::stdout_suppressed() {
            return;
        }
        for chunk in assistant_chunks(text) {
            if !chunk.is_empty() {
                crate::output::print_stdout_line(WHO_M, chunk);
            }
        }
    }

    pub fn mini_assistant(&self, text: &str) {
        self.stream_assistant_chunks(text);
    }
}

fn assistant_chunks(text: &str) -> Vec<&str> {
    if text.is_empty() {
        return vec![];
    }
    if !text.contains('\n') {
        return vec![text];
    }
    text.split('\n').filter(|s| !s.is_empty()).collect()
}

/// Legacy helper kept for trace unit tests.
#[allow(dead_code)]
pub(crate) fn format_mini_bash_tool_line(command: &str, exit_code: i32, elapsed: Duration) -> String {
    format_classified_tool_line(
        classify_bash_command(command),
        command,
        exit_code,
        elapsed,
    )
}
