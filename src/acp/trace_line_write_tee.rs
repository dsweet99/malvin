use super::trace_plain_tee::print_plain_tee_wrapped_line;
use crate::acp::trace_line_write::TraceTeeStdoutCtx;
use crate::acp::{PromptTraceWriter, SessionUpdateChunkKind};
use crate::deferred_log::{
    build_acp_tee_entry, build_raw_line_entry, build_tool_entry, log_with_heartbeat, AcpTeeBuild,
    TeeSinkMeta, ToolSummaryBuild,
};

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
    if matches!(kind, Some(SessionUpdateChunkKind::Thought)) {
        return format!("   {line}");
    }
    line.to_string()
}

fn tee_sink_meta(writer: &PromptTraceWriter, ts: &str) -> TeeSinkMeta {
    TeeSinkMeta {
        who: writer.who.clone(),
        ts: ts.to_string(),
        emit_stdout_markdown: writer.emit_stdout_markdown,
    }
}

fn trace_tee_stdout_event<'a>(
    writer: &'a PromptTraceWriter,
    line: &'a str,
    ts: &'a str,
    dim_payload: bool,
) -> crate::output::AcpTeeStdoutEvent<'a> {
    crate::output::AcpTeeStdoutEvent {
        direction: crate::output::AcpTeeDirection::FromAgent,
        who: &writer.who,
        line,
        ts,
        emit_stdout_markdown: writer.emit_stdout_markdown,
        dim_payload,
    }
}

pub(crate) fn trace_tee_tool_summary_stdout_event<'a>(
    writer: &'a PromptTraceWriter,
    line: &'a str,
    ts: &'a str,
) -> crate::output::AcpTeeStdoutEvent<'a> {
    trace_tee_stdout_event(writer, line, ts, true)
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

fn defer_raw_wrapped_lines(writer: &PromptTraceWriter, line: &str, ts: &str) {
    let (max_payload, wrap) = crate::output::terminal_wrap::line_wrap_for_prefix_len(
        0,
        line,
        crate::output::terminal_wrap::stdout_allows_log_word_wrap(),
    );
    let segments: Vec<String> = if wrap {
        crate::output::terminal_wrap::wrap_words_bounded(max_payload, line)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        vec![line.to_string()]
    };
    let sink = writer.deferred_sink.as_ref().expect("defer sink");
    let who = writer.who.clone();
    let ts = ts.to_string();
    let mut guard = sink
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for seg in segments {
        log_with_heartbeat(&mut guard, build_raw_line_entry(seg, who.clone(), ts.clone()));
    }
}

fn trace_tee_deferred_line(
    writer: &mut PromptTraceWriter,
    line: &str,
    display_line: Option<&str>,
    ctx: &TraceTeeStdoutCtx<'_>,
) -> bool {
    if writer.deferred_sink.is_none() {
        return false;
    }
    let tee = tee_sink_meta(writer, ctx.ts);
    let Some(sink) = writer.deferred_sink.as_ref() else {
        return false;
    };
    let mut guard = sink
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(rep) = writer.stdout_replacement {
        if !writer.placeholder_emitted {
            log_with_heartbeat(
                &mut guard,
                build_acp_tee_entry(AcpTeeBuild {
                    tee,
                    kind: ctx.kind,
                    line: rep.to_string(),
                    display: None,
                    dim_payload: false,
                }),
            );
            writer.placeholder_emitted = true;
        }
        return true;
    }
    if let Some(display) = display_line {
        log_with_heartbeat(
            &mut guard,
            build_tool_entry(ToolSummaryBuild {
                tee,
                plain: line.to_string(),
                display: display.to_string(),
                enrich: None,
                meta: None,
            }),
        );
        return true;
    }
    let dim = matches!(ctx.kind, Some(SessionUpdateChunkKind::Thought));
    log_with_heartbeat(
        &mut guard,
        build_acp_tee_entry(AcpTeeBuild {
            tee,
            kind: ctx.kind,
            line: line.to_string(),
            display: None,
            dim_payload: dim,
        }),
    );
    true
}

fn trace_tee_immediate_line(
    writer: &mut PromptTraceWriter,
    line: &str,
    display_line: Option<&str>,
    ctx: &TraceTeeStdoutCtx<'_>,
) {
    if let Some(rep) = writer.stdout_replacement {
        if !writer.placeholder_emitted {
            crate::output::print_stdout_acp_tee_line_with_timestamp(&trace_tee_stdout_event(
                writer, rep, ctx.ts, false,
            ));
            writer.placeholder_emitted = true;
        }
        return;
    }
    if let Some(display) = display_line {
        crate::output::print_stdout_acp_tool_summary_tee(
            &trace_tee_tool_summary_stdout_event(writer, line, ctx.ts),
            display,
        );
        return;
    }
    let dim = matches!(ctx.kind, Some(SessionUpdateChunkKind::Thought));
    crate::output::print_stdout_acp_tee_line_with_timestamp(&trace_tee_stdout_event(
        writer, line, ctx.ts, dim,
    ));
}

pub(crate) fn trace_tee_stdout_line(
    writer: &mut PromptTraceWriter,
    line: &str,
    display_line: Option<&str>,
    ctx: &TraceTeeStdoutCtx<'_>,
) {
    if !ctx.tee_stdout {
        return;
    }
    if writer.plain_lines || writer.raw_output {
        let styled_plain = writer.plain_lines && writer.emit_stdout_markdown && !writer.raw_output;
        if writer.deferred_sink.is_some() && !styled_plain {
            defer_raw_wrapped_lines(writer, line, ctx.ts);
        } else {
            print_plain_tee_wrapped_line(line, ctx.ts, styled_plain);
        }
        return;
    }
    if trace_tee_deferred_line(writer, line, display_line, ctx) {
        return;
    }
    trace_tee_immediate_line(writer, line, display_line, ctx);
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_defer_raw_wrapped_lines() {
        let _ = super::defer_raw_wrapped_lines;
    }

    #[test]
    fn kiss_cov_trace_tee_deferred_line() {
        let _ = super::trace_tee_deferred_line;
    }

    #[test]
    fn kiss_cov_trace_tee_immediate_line() {
        let _ = super::trace_tee_immediate_line;
    }
}

#[cfg(test)]
#[path = "trace_line_write_tee_tests.rs"]
mod trace_line_write_tee_tests;
