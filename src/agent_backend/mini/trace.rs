//! Mini trace events to `trace.jsonl` and stdout.
//!
//! Dual-contract routing: see [`crate::observability`] for channel trust rules.
//! Audit writes go through [`emit_audit`]; narrative writes through [`emit_narrative`].

use std::time::Duration;

use malvin_mini::ResponseUsage;

use super::acp_trace_shim::{
    append_out_raw, emit_agent_message_chunk, emit_agent_thought_chunk, emit_bash_tool_call,
    emit_llm_usage, emit_mini_http_exchange, trace_for_run_dir, MiniHttpExchangeRecord,
};
use crate::acp::AgentIoOptions;
use crate::acp_trace_impersonation::SyntheticAcpSessionUpdate;
use crate::observability::{narrative_suppressed, AUDIT_CHANNEL, NARRATIVE_CHANNEL, ObservabilityChannel};
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

/// Channel target for [`emit_audit`] and audit-only public methods.
pub const MINI_AUDIT_CHANNEL: ObservabilityChannel = AUDIT_CHANNEL;
/// Channel target for [`emit_narrative`] and dual-emission methods.
pub const MINI_NARRATIVE_CHANNEL: ObservabilityChannel = NARRATIVE_CHANNEL;

impl MiniTraceSink {
    #[must_use]
    pub const fn new(run_dir: Option<std::path::PathBuf>, io: AgentIoOptions) -> Self {
        Self {
            run_dir,
            io,
            plain_lines: false,
        }
    }

    pub(crate) fn acp_trace_for_audit(&self) -> Option<crate::acp::AcpJsonlTrace> {
        self.run_dir
            .as_ref()
            .map(|dir| trace_for_run_dir(dir.as_path()))
    }

    /// Audit-only: outgoing prompt body (`out` direction).
    pub fn log_outgoing_prompt(&self, text: &str) {
        emit_audit(self, SyntheticAcpSessionUpdate::OutRaw, |trace| append_out_raw(trace, text));
    }

    /// Audit-only: LLM token usage (`miniUsage` extension).
    pub fn mini_llm_request(&self, usage: Option<&ResponseUsage>) {
        if let Some(u) = usage {
            emit_audit(self, SyntheticAcpSessionUpdate::LlmUsage, |trace| emit_llm_usage(trace, u));
        }
    }

    /// Dual: audit `agent_thought_chunk`; narrative `b|` when `show_thoughts_on_stdout`.
    pub fn mini_thought(&self, text: &str) {
        if text.is_empty() {
            return;
        }
        emit_audit(self, SyntheticAcpSessionUpdate::AgentThoughtChunk, |trace| {
            emit_agent_thought_chunk(trace, text);
        });
        if !self.io.show_thoughts_on_stdout {
            return;
        }
        for chunk in assistant_chunks(text) {
            emit_narrative(self, WHO_B, chunk);
        }
    }

    /// Dual: audit `tool_call`; narrative `t|` summary unless `plain_lines` / `raw_output`.
    pub fn mini_bash_exec(
        &self,
        command: &str,
        exit_code: i32,
        elapsed: Duration,
        comment: Option<&str>,
    ) {
        let kind = classify_bash_command(command);
        emit_audit(self, SyntheticAcpSessionUpdate::ToolCall, |trace| {
            emit_bash_tool_call(trace, bash_kind_wire_name(kind), command, exit_code);
        });
        if self.plain_lines || self.io.raw_output {
            return;
        }
        if mini_narrative_suppressed(self) {
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

    /// Audit-only: assistant text chunks (`agent_message_chunk`).
    pub fn record_assistant_audit(&self, text: &str) {
        emit_audit(self, SyntheticAcpSessionUpdate::AgentMessageChunk, |trace| {
            for chunk in assistant_chunks(text) {
                emit_agent_message_chunk(trace, chunk);
            }
        });
    }

    /// Dual: audit + narrative assistant chunks (`m|` or plain untagged when `plain_lines`).
    pub fn stream_assistant_chunks(&self, text: &str) {
        self.record_assistant_audit(text);
        for chunk in assistant_chunks(text) {
            emit_narrative(self, WHO_M, chunk);
        }
    }

    pub fn mini_assistant_with_reasoning(&self, content: &str, reasoning: Option<&str>) {
        if let Some(r) = reasoning {
            self.mini_thought(r);
        }
        self.stream_assistant_chunks(content);
    }
}

/// Write one audit-only event to `trace.jsonl`. No narrative emission.
pub(crate) fn emit_audit(
    sink: &MiniTraceSink,
    _kind: SyntheticAcpSessionUpdate,
    write: impl FnOnce(&crate::acp::AcpJsonlTrace),
) {
    assert!(matches!(MINI_AUDIT_CHANNEL, ObservabilityChannel::Audit));
    if let Some(trace) = sink.acp_trace_for_audit() {
        write(&trace);
    }
}

fn mini_narrative_suppressed(sink: &MiniTraceSink) -> bool {
    narrative_suppressed(sink.io.no_tee)
}

/// Write one narrative line to stdout / `stdout.log`, respecting suppression flags.
pub(crate) fn emit_narrative(sink: &MiniTraceSink, who: &str, chunk: &str) {
    assert!(matches!(MINI_NARRATIVE_CHANNEL, ObservabilityChannel::Narrative));
    if chunk.is_empty() || mini_narrative_suppressed(sink) {
        return;
    }
    if sink.plain_lines || sink.io.raw_output {
        let ts = crate::output::timestamp_now_string();
        crate::acp::print_plain_tee_wrapped_line(
            chunk,
            &ts,
            sink.io.emit_stdout_markdown && !sink.io.raw_output,
        );
    } else {
        crate::output::print_stdout_line(who, chunk);
    }
}

pub fn record_http_exchange(sink: &MiniTraceSink, record: MiniHttpExchangeRecord<'_>) {
    emit_audit(sink, SyntheticAcpSessionUpdate::MiniHttpExchange, |trace| {
        emit_mini_http_exchange(trace, record);
    });
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
