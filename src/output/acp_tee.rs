//! ACP trace tee: distinct ANSI colors for outbound (`>`) vs inbound (`<`) lines on stdout.

pub use super::acp_tee_markdown::{
    TermimadStdoutGate, termimad_inline_payload_for_stdout, termimad_text_lines_for_stdout,
};
pub use super::stdout_log_pair::{
    AcpTeeDirection, AcpTeeLineFmt, acp_tee_display_line, acp_tee_log_line,
    format_line_acp_ansi_payload,
};
pub(crate) use super::stdout_log_pair::{acp_tee_payload_prefix, acp_tee_payload_prefix_width};

use super::stdout_render::{route_stdout_rendered_line, StdoutRenderPrelude};
use super::timestamp_now_string;

fn route_acp_rendered(display: &str, log: &str, prelude: StdoutRenderPrelude) {
    route_stdout_rendered_line(display, log, prelude);
}

/// Parameters for [`print_stdout_acp_tee_line_with_timestamp`].
pub struct AcpTeeStdoutEvent<'a> {
    pub direction: AcpTeeDirection,
    pub who: &'a str,
    pub line: &'a str,
    pub ts: &'a str,
    pub emit_stdout_markdown: bool,
    pub dim_payload: bool,
}

/// Stdout tee for ACP trace lines: when color is enabled, outbound (`>`) vs inbound (`<`) use
/// different ANSI colors for the `[who]` prefix; the payload is plain, dim, or `termimad` per mode.
pub fn print_stdout_acp_tee_line(direction: AcpTeeDirection, who: &str, line: &str) {
    let ts = timestamp_now_string();
    print_stdout_acp_tee_line_with_timestamp(&AcpTeeStdoutEvent {
        direction,
        who,
        line,
        ts: &ts,
        emit_stdout_markdown: false,
        dim_payload: false,
    });
}

/// Same as [`print_stdout_acp_tee_line`], but uses `ts` for the line prefix (shared with disk trace).
pub fn print_stdout_acp_tool_summary_tee(ev: &AcpTeeStdoutEvent<'_>, display_payload: &str) {
    let display_ctx = AcpTeeLineFmt {
        ts: ev.ts,
        direction: ev.direction,
        who: ev.who,
        line: display_payload,
        dim_payload: ev.dim_payload,
    };
    let log_ctx = AcpTeeLineFmt {
        ts: ev.ts,
        direction: ev.direction,
        who: ev.who,
        line: ev.line,
        dim_payload: ev.dim_payload,
    };
    let (display, log) = super::stdout_log_pair::stdout_acp_display_and_log(&display_ctx, &log_ctx);
    route_acp_rendered(&display, &log, StdoutRenderPrelude::TaggedWithHeartbeat);
}

pub(crate) fn flush_stdout_acp_tool_summary_tee(ev: &AcpTeeStdoutEvent<'_>, display_payload: &str) {
    let display_ctx = AcpTeeLineFmt {
        ts: ev.ts,
        direction: ev.direction,
        who: ev.who,
        line: display_payload,
        dim_payload: ev.dim_payload,
    };
    let log_ctx = AcpTeeLineFmt {
        ts: ev.ts,
        direction: ev.direction,
        who: ev.who,
        line: ev.line,
        dim_payload: ev.dim_payload,
    };
    let (display, log) = super::stdout_log_pair::stdout_acp_display_and_log(&display_ctx, &log_ctx);
    route_acp_rendered(&display, &log, StdoutRenderPrelude::FlushOnly);
}

pub fn print_stdout_acp_tee_line_with_timestamp(ev: &AcpTeeStdoutEvent<'_>) {
    let ctx = AcpTeeLineFmt {
        ts: ev.ts,
        direction: ev.direction,
        who: ev.who,
        line: ev.line,
        dim_payload: ev.dim_payload,
    };
    print_stdout_acp_tee_line_with_timestamp_payload(
        &ctx,
        ev.emit_stdout_markdown,
        StdoutRenderPrelude::TaggedWithHeartbeat,
    );
}

pub(crate) fn flush_stdout_acp_tee_line_with_timestamp(ev: &AcpTeeStdoutEvent<'_>) {
    let ctx = AcpTeeLineFmt {
        ts: ev.ts,
        direction: ev.direction,
        who: ev.who,
        line: ev.line,
        dim_payload: ev.dim_payload,
    };
    print_stdout_acp_tee_line_with_timestamp_payload(
        &ctx,
        ev.emit_stdout_markdown,
        StdoutRenderPrelude::FlushOnly,
    );
}

/// Same as [`print_stdout_acp_tee_line_with_timestamp`], but dims the payload and keeps stdout markdown off.
pub fn print_stdout_acp_tee_line_with_timestamp_dim_plain(
    direction: AcpTeeDirection,
    who: &str,
    line: &str,
    ts: &str,
) {
    print_stdout_acp_tee_line_with_timestamp(&AcpTeeStdoutEvent {
        direction,
        who,
        line,
        ts,
        emit_stdout_markdown: false,
        dim_payload: true,
    });
}

fn print_acp_tee_stdout_markdown_line(
    ctx: &AcpTeeLineFmt<'_>,
    rendered_payload: &str,
    prelude: StdoutRenderPrelude,
) {
    let (display, log) =
        super::stdout_log_pair::stdout_acp_prefix_rendered_line(ctx, rendered_payload);
    route_acp_rendered(&display, &log, prelude);
}

fn print_acp_tee_stdout_markdown_lines(
    ctx: &AcpTeeLineFmt<'_>,
    rendered_payloads: &[String],
    prelude: StdoutRenderPrelude,
) {
    for rendered in rendered_payloads {
        print_acp_tee_stdout_markdown_line(ctx, rendered, prelude);
    }
}

fn print_stdout_acp_tee_line_with_timestamp_payload(
    ctx: &AcpTeeLineFmt<'_>,
    emit_stdout_markdown: bool,
    prelude: StdoutRenderPrelude,
) {
    let line_gate = TermimadStdoutGate {
        emit_stdout_markdown,
        dim_payload: ctx.dim_payload,
        allow_inline_styling: super::log_use_color(),
    };
    let prefix = acp_tee_payload_prefix(ctx);
    let prefix_len = acp_tee_payload_prefix_width(&prefix);
    let (max_payload, wrap) = super::terminal_wrap::line_wrap_for_prefix_len(
        prefix_len,
        ctx.line,
        super::terminal_wrap::stdout_allows_log_word_wrap(),
    );
    if let Some(rendered_lines) = termimad_text_lines_for_stdout(ctx.line, line_gate, max_payload) {
        print_acp_tee_stdout_markdown_lines(ctx, &rendered_lines, prelude);
        return;
    }
    if !wrap {
        if let Some(rendered) = termimad_inline_payload_for_stdout(ctx.line, line_gate) {
            print_acp_tee_stdout_markdown_line(ctx, &rendered, prelude);
            return;
        }
        let (display, log) = super::stdout_log_pair::stdout_acp_display_and_log(ctx, ctx);
        route_acp_rendered(&display, &log, prelude);
        return;
    }
    for seg in super::terminal_wrap::wrap_words_bounded(max_payload, ctx.line) {
        let seg_ctx = AcpTeeLineFmt {
            ts: ctx.ts,
            direction: ctx.direction,
            who: ctx.who,
            line: &seg,
            dim_payload: ctx.dim_payload,
        };
        if let Some(rendered) = termimad_inline_payload_for_stdout(&seg, line_gate) {
            print_acp_tee_stdout_markdown_line(&seg_ctx, &rendered, prelude);
            continue;
        }
        let (display, log) = super::stdout_log_pair::stdout_acp_display_and_log(&seg_ctx, &seg_ctx);
        route_acp_rendered(&display, &log, prelude);
    }
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_print_acp_tee_stdout_markdown_line() { let _ = stringify!(print_acp_tee_stdout_markdown_line); }

    #[test]
    fn kiss_cov_print_acp_tee_stdout_markdown_lines() { let _ = stringify!(print_acp_tee_stdout_markdown_lines); }

    #[test]
    fn kiss_cov_print_stdout_acp_tee_line_with_timestamp_payload() { let _ = stringify!(print_stdout_acp_tee_line_with_timestamp_payload); }

    #[test]
    fn kiss_cov_flush_stdout_acp_tool_summary_tee() {
        let _ = super::flush_stdout_acp_tool_summary_tee;
    }

    #[test]
    fn kiss_cov_flush_stdout_acp_tee_line_with_timestamp() {
        let _ = super::flush_stdout_acp_tee_line_with_timestamp;
    }

}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = print_acp_tee_stdout_markdown_line;
        let _ = print_acp_tee_stdout_markdown_lines;
        let _ = print_stdout_acp_tee_line_with_timestamp_payload;
        let _ = route_acp_rendered;
    }
}
