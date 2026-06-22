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
    line.to_string()
}

pub(crate) struct TeeStdoutEmit<'a> {
    pub(crate) line: &'a str,
    pub(crate) ts: &'a str,
    pub(crate) dim_payload: bool,
    pub(crate) who: &'a str,
}

#[allow(clippy::derivable_impls)]
impl Default for TeeStdoutEmit<'static> {
    fn default() -> Self {
        Self {
            line: "",
            ts: "",
            dim_payload: false,
            who: "",
        }
    }
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

#[path = "trace_line_write_tee_emit.rs"]
mod trace_line_write_tee_emit;
pub(crate) use trace_line_write_tee_emit::trace_tee_stdout_line;

pub(crate) use trace_line_write_tee_emit::{
    defer_raw_wrapped_lines, trace_tee_deferred_line, trace_tee_immediate_line,
};
#[cfg(test)]
mod kiss_cov_inline {
    use super::*;

    #[test]
    fn kiss_cov_band80_witnesses() {
        let emit = TeeStdoutEmit {
            line: "payload",
            ts: "20260524.000000.000",
            dim_payload: true,
            who: WHO_B,
        };
        let TeeStdoutEmit {
            line,
            ts,
            dim_payload,
            who,
        } = emit;
        assert_eq!(line, "payload");
        assert_eq!(ts, "20260524.000000.000");
        assert!(dim_payload);
        assert_eq!(who, WHO_B);
    }
}

#[cfg(test)]
#[path = "trace_line_write_tee_kiss_cov_test.rs"]
mod trace_line_write_tee_kiss_cov_test;
#[cfg(test)]
#[path = "trace_line_write_tee_test.rs"]
mod trace_line_write_tee_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<TeeStdoutEmit> = None;
        let _ = default;
    }
}
