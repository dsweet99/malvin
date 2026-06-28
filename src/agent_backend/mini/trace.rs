//! Mini trace events to `trace.jsonl` and stdout.

use std::time::Duration;

use malvin_mini::ResponseUsage;

use super::acp_trace_shim::{
    append_out_raw, emit_agent_message_chunk, emit_agent_thought_chunk, emit_bash_tool_call,
    emit_llm_usage, trace_for_run_dir,
};
use crate::acp::AgentIoOptions;
use crate::output::{AcpTeeDirection, AcpTeeStdoutEvent, WHO_B, WHO_M, WHO_T};
use crate::tool_summary::{
    bash_kind_wire_name, classify_bash_command, format_classified_tool_line,
    tool_summary_stdout_display, ClassifiedToolLineInput,
};

pub struct MiniTraceSink {
    pub run_dir: Option<std::path::PathBuf>,
    pub io: AgentIoOptions,
    /// When true, stdout matches ACP `do` (`plain_lines`): unprefixed assistant text, no tool tee.
    pub plain_lines: bool,
}

impl MiniTraceSink {
    #[must_use]
    pub const fn new(run_dir: Option<std::path::PathBuf>, io: AgentIoOptions) -> Self {
        Self {
            run_dir,
            io,
            plain_lines: false,
        }
    }

    fn tee_assistant_chunk(&self, who: &str, chunk: &str) {
        if self.plain_lines || self.io.raw_output {
            let ts = crate::output::timestamp_now_string();
            crate::acp::print_plain_tee_wrapped_line(
                chunk,
                &ts,
                self.io.emit_stdout_markdown && !self.io.raw_output,
            );
        } else {
            crate::output::print_stdout_line(who, chunk);
        }
    }

    pub(crate) fn acp_trace_for_audit(&self) -> Option<crate::acp::AcpJsonlTrace> {
        self.run_dir
            .as_ref()
            .map(|dir| trace_for_run_dir(dir.as_path()))
    }

    pub fn log_outgoing_prompt(&self, text: &str) {
        if let Some(trace) = self.acp_trace_for_audit() {
            append_out_raw(&trace, text);
        }
    }

    pub fn mini_llm_request(&self, usage: Option<&ResponseUsage>) {
        if let (Some(trace), Some(u)) = (self.acp_trace_for_audit(), usage) {
            emit_llm_usage(&trace, u);
        }
    }

    pub fn mini_thought(&self, text: &str) {
        if text.is_empty() {
            return;
        }
        if let Some(trace) = self.acp_trace_for_audit() {
            emit_agent_thought_chunk(&trace, text);
        }
        if self.io.no_tee || crate::output::stdout_suppressed() {
            return;
        }
        if !self.io.show_thoughts_on_stdout {
            return;
        }
        for chunk in assistant_chunks(text) {
            if !chunk.is_empty() {
                self.tee_assistant_chunk(WHO_B, chunk);
            }
        }
    }

    pub fn mini_bash_exec(
        &self,
        command: &str,
        exit_code: i32,
        elapsed: Duration,
        comment: Option<&str>,
    ) {
        let kind = classify_bash_command(command);
        if let Some(trace) = self.acp_trace_for_audit() {
            emit_bash_tool_call(&trace, bash_kind_wire_name(kind), command, exit_code);
        }
        if self.io.no_tee
            || crate::output::stdout_suppressed()
            || self.plain_lines
            || self.io.raw_output
        {
            return;
        }
        let plain = format_classified_tool_line(ClassifiedToolLineInput {
            kind,
            command,
            exit_code,
            elapsed,
            comment,
        });
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
        if let Some(trace) = self.acp_trace_for_audit() {
            for chunk in assistant_chunks(text) {
                emit_agent_message_chunk(&trace, chunk);
            }
        }
        if self.io.no_tee || crate::output::stdout_suppressed() {
            return;
        }
        for chunk in assistant_chunks(text) {
            if !chunk.is_empty() {
                self.tee_assistant_chunk(WHO_M, chunk);
            }
        }
    }

    pub fn mini_assistant_with_reasoning(&self, content: &str, reasoning: Option<&str>) {
        if let Some(r) = reasoning {
            self.mini_thought(r);
        }
        self.stream_assistant_chunks(content);
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
pub(crate) fn format_mini_bash_tool_line(
    command: &str,
    exit_code: i32,
    elapsed: Duration,
    comment: Option<&str>,
) -> String {
    format_classified_tool_line(ClassifiedToolLineInput {
        kind: classify_bash_command(command),
        command,
        exit_code,
        elapsed,
        comment,
    })
}
