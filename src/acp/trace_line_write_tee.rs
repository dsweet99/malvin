use crate::acp::trace_line_write::TraceTeeStdoutCtx;
use crate::acp::{PromptTraceWriter, SessionUpdateChunkKind};
use crate::output::{WHO_B, WHO_M, WHO_T};

pub(crate) fn format_trace_display_line(line: &str, kind: Option<SessionUpdateChunkKind>) -> String {
    match kind {
        Some(SessionUpdateChunkKind::Thought) => format!("[{line}]"),
        _ => line.to_string(),
    }
}

pub(crate) fn trace_stdout_tee_payload(
    line: &str,
    kind: Option<SessionUpdateChunkKind>,
    writer: &PromptTraceWriter,
) -> String {
    if writer.plain_lines || writer.raw_output {
        return line.to_string();
    }
    let _ = kind;
    line.to_string()
}

pub(crate) struct TeeStdoutEmit<'a> {
    pub(crate) line: &'a str,
    pub(crate) ts: &'a str,
    pub(crate) dim_payload: bool,
    pub(crate) who: &'a str,
}

pub(crate) fn from_agent_who(ctx: &TraceTeeStdoutCtx<'_>, is_tool: bool) -> String {
    if is_tool {
        WHO_T.to_string()
    } else if matches!(ctx.kind, Some(SessionUpdateChunkKind::Thought)) {
        WHO_B.to_string()
    } else {
        WHO_M.to_string()
    }
}

pub(crate) const fn trace_tee_stdout_event<'a>(
    writer: &'a PromptTraceWriter,
    emit: TeeStdoutEmit<'a>,
) -> crate::output::AcpTeeStdoutEvent<'a> {
    crate::output::AcpTeeStdoutEvent {
        direction: crate::output::AcpTeeDirection::FromAgent,
        who: emit.who,
        line: emit.line,
        ts: emit.ts,
        emit_stdout_markdown: writer.emit_stdout_markdown,
        dim_payload: emit.dim_payload,
    }
}

pub(crate) const fn trace_tee_tool_summary_stdout_event<'a>(
    writer: &'a PromptTraceWriter,
    line: &'a str,
    ts: &'a str,
) -> crate::output::AcpTeeStdoutEvent<'a> {
    trace_tee_stdout_event(
        writer,
        TeeStdoutEmit {
            line,
            ts,
            dim_payload: true,
            who: WHO_T,
        },
    )
}

#[cfg(test)]
pub(crate) fn format_styled_tool_summary_tee_line(
    writer: &PromptTraceWriter,
    plain_bracketed: &str,
    display: &str,
    ts: &str,
) -> String {
    let ev = trace_tee_tool_summary_stdout_event(writer, plain_bracketed, ts);
    assert!(ev.dim_payload, "tool-summary tee must dim payload");
    crate::output::format_line_acp_ansi_payload(&crate::output::AcpTeeLineFmt {
        ts: ev.ts,
        direction: crate::output::AcpTeeDirection::FromAgent,
        who: ev.who,
        line: display,
        dim_payload: ev.dim_payload,
    })
}

#[path = "observability_emit.rs"]
mod observability_emit;
#[path = "trace_line_write_tee_emit.rs"]
mod trace_line_write_tee_emit;
pub(crate) use observability_emit::write_audit_trace_line;
pub(crate) use observability_emit::tee_narrative_line as trace_tee_stdout_line;

#[cfg(test)]
pub(crate) use trace_line_write_tee_emit::{
    defer_raw_wrapped_lines, trace_tee_deferred_line, trace_tee_immediate_line,
};

#[cfg(test)]
#[path = "trace_line_write_tee_tests.rs"]
mod trace_line_write_tee_tests;
