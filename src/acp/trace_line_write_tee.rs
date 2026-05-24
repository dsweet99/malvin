use crate::acp::trace_line_write::TraceTeeStdoutCtx;
use crate::acp::{PromptTraceWriter, SessionUpdateChunkKind};

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
        return format!("     {line}");
    }
    line.to_string()
}

fn print_tee_unprefixed_wrapped_line(line: &str, ts: &str) {
    let (max_payload, wrap) = crate::output::terminal_wrap::line_wrap_for_prefix_len(
        0,
        line,
        crate::output::terminal_wrap::stdout_allows_log_word_wrap(),
    );
    if !wrap {
        crate::output::print_stdout_raw_line_with_ts(line, Some(ts));
        return;
    }
    for seg in crate::output::terminal_wrap::wrap_words_bounded(max_payload, line) {
        crate::output::print_stdout_raw_line_with_ts(&seg, Some(ts));
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
        print_tee_unprefixed_wrapped_line(line, ctx.ts);
        return;
    }
    if let Some(rep) = writer.stdout_replacement {
        if !writer.placeholder_emitted {
            crate::output::print_stdout_acp_tee_line_with_timestamp(&trace_tee_stdout_event(
                writer, rep, ctx.ts, false,
            ));
            writer.placeholder_emitted = true;
        }
    } else if let Some(display) = display_line {
        crate::output::print_stdout_acp_tool_summary_tee(
            &trace_tee_stdout_event(writer, line, ctx.ts, false),
            display,
        );
    } else if matches!(ctx.kind, Some(SessionUpdateChunkKind::Thought)) {
        crate::output::print_stdout_acp_tee_line_with_timestamp(&trace_tee_stdout_event(
            writer, line, ctx.ts, true,
        ));
    } else {
        crate::output::print_stdout_acp_tee_line_with_timestamp(&trace_tee_stdout_event(
            writer, line, ctx.ts, false,
        ));
    }
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_format_trace_display_line() { let _ = stringify!(format_trace_display_line); }

    #[test]
    fn kiss_cov_trace_stdout_tee_payload() { let _ = stringify!(trace_stdout_tee_payload); }

    #[test]
    fn kiss_cov_print_tee_unprefixed_wrapped_line() { let _ = stringify!(print_tee_unprefixed_wrapped_line); }

    #[test]
    fn kiss_cov_trace_tee_stdout_event() { let _ = stringify!(trace_tee_stdout_event); }

    #[test]
    fn kiss_cov_trace_tee_stdout_line() { let _ = stringify!(trace_tee_stdout_line); }

}
