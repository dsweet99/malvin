//! Deferred and immediate narrative tee emission for ACP trace lines.
//!
//! Audit records stay on the wire/`trace.jsonl` path; this module tees to the narrative channel.
//! See [`crate::observability`] for trust rules.
use super::{from_agent_who, trace_tee_stdout_event, trace_tee_tool_summary_stdout_event, TeeStdoutEmit};
use crate::acp::trace_line_write::TraceTeeStdoutCtx;
use crate::acp::trace_plain_tee::print_plain_tee_wrapped_line;
use crate::acp::{PromptTraceWriter, SessionUpdateChunkKind};
use crate::deferred_log::{
    build_acp_tee_entry, build_raw_line_entry, build_tool_entry, log_with_heartbeat, AcpTeeBuild,
    TeeSinkMeta, ToolSummaryBuild,
};
use crate::output::{WHO_M, WHO_T};

pub(crate) fn defer_raw_wrapped_lines(writer: &PromptTraceWriter, line: &str, ts: &str) {
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

pub(crate) fn trace_tee_deferred_line(
    writer: &mut PromptTraceWriter,
    line: &str,
    display_line: Option<&str>,
    ctx: &TraceTeeStdoutCtx<'_>,
) -> bool {
    if writer.deferred_sink.is_none() {
        return false;
    }
    let tee = TeeSinkMeta {
        who: if display_line.is_some() {
            WHO_T.to_string()
        } else {
            from_agent_who(ctx, false)
        },
        ts: ctx.ts.to_string(),
        emit_stdout_markdown: writer.emit_stdout_markdown,
    };
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

pub(crate) fn trace_tee_immediate_line(
    writer: &mut PromptTraceWriter,
    line: &str,
    display_line: Option<&str>,
    ctx: &TraceTeeStdoutCtx<'_>,
) {
    if let Some(rep) = writer.stdout_replacement {
        if !writer.placeholder_emitted {
            crate::output::print_stdout_acp_tee_line_with_timestamp(&trace_tee_stdout_event(
                writer,
                TeeStdoutEmit {
                    line: rep,
                    ts: ctx.ts,
                    dim_payload: false,
                    who: WHO_M,
                },
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
    let who = from_agent_who(ctx, false);
    crate::output::print_stdout_acp_tee_line_with_timestamp(&trace_tee_stdout_event(
        writer,
        TeeStdoutEmit {
            line,
            ts: ctx.ts,
            dim_payload: dim,
            who: &who,
        },
    ));
}

pub(crate) fn tee_narrative_line_impl(
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
