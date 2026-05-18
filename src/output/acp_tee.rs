//! ACP trace tee: distinct ANSI colors for outbound (`>`) vs inbound (`<`) lines on stdout.

pub use super::acp_tee_markdown::{
    TermimadStdoutGate, termimad_inline_payload_for_stdout, termimad_text_lines_for_stdout,
};
use super::{ANSI_DIM, ANSI_RESET};
use super::{format_log_tag_inner, stdout_use_color, timestamp_now_string};

use crate::ansi_strip::strip_ansi_escapes;
use unicode_width::UnicodeWidthStr;

const ANSI_BRIGHT_GREEN: &str = "\x1b[92m";
const ANSI_BRIGHT_MAGENTA: &str = "\x1b[95m";

/// Tee direction for ACP trace lines echoed to stdout (distinct ANSI bracket colors on TTY).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcpTeeDirection {
    /// Lines sent to the agent (`>` trace tag, `session/prompt` body).
    ToAgent,
    /// Lines streamed from the agent (`<` trace tag, ACP reader).
    FromAgent,
}

pub struct AcpTeeLineFmt<'a> {
    pub ts: &'a str,
    pub direction: AcpTeeDirection,
    pub who: &'a str,
    pub line: &'a str,
    pub dim_payload: bool,
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

/// ANSI ACP tee line prefix (outbound vs inbound bracket colors).
///
/// Differs from [`super::format_line_with_timestamp_ansi`] (default cyan `who`). Prefer
/// [`print_stdout_acp_tee_line`] for stdout.
#[must_use]
pub fn format_line_with_timestamp_acp_ansi(
    ts: &str,
    direction: AcpTeeDirection,
    who: &str,
    line: &str,
) -> String {
    format_line_with_timestamp_acp_ansi_payload(&AcpTeeLineFmt {
        ts,
        direction,
        who,
        line,
        dim_payload: false,
    })
}

#[must_use]
pub fn format_line_with_timestamp_acp_ansi_payload(ctx: &AcpTeeLineFmt<'_>) -> String {
    let inner = format_log_tag_inner(ctx.who);
    let bracket = match ctx.direction {
        AcpTeeDirection::ToAgent => ANSI_BRIGHT_GREEN,
        AcpTeeDirection::FromAgent => ANSI_BRIGHT_MAGENTA,
    };
    if ctx.dim_payload {
        format!(
            "{ANSI_DIM}{}{ANSI_RESET}{bracket}:[{inner}]:{ANSI_RESET} {ANSI_DIM}{}{ANSI_RESET}",
            ctx.ts, ctx.line
        )
    } else {
        format!(
            "{ANSI_DIM}{}{ANSI_RESET}{bracket}:[{inner}]:{ANSI_RESET} {}",
            ctx.ts, ctx.line
        )
    }
}

/// Stdout tee for ACP trace lines: when color is enabled, outbound (`>`) vs inbound (`<`) use
/// different ANSI colors for the `[who]:` prefix; the payload is plain, dim, or `termimad` per mode.
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
pub fn print_stdout_acp_tee_line_with_timestamp(ev: &AcpTeeStdoutEvent<'_>) {
    let ctx = AcpTeeLineFmt {
        ts: ev.ts,
        direction: ev.direction,
        who: ev.who,
        line: ev.line,
        dim_payload: ev.dim_payload,
    };
    print_stdout_acp_tee_line_with_timestamp_payload(&ctx, ev.emit_stdout_markdown);
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

fn acp_tee_payload_prefix(ctx: &AcpTeeLineFmt<'_>) -> String {
    let empty = AcpTeeLineFmt {
        ts: ctx.ts,
        direction: ctx.direction,
        who: ctx.who,
        line: "",
        dim_payload: ctx.dim_payload,
    };
    if super::stdout_use_color() {
        format_line_with_timestamp_acp_ansi_payload(&empty)
    } else {
        super::format_line_with_timestamp(ctx.ts, ctx.who, "")
    }
}

fn acp_tee_payload_prefix_width(prefix: &str) -> usize {
    strip_ansi_escapes(prefix).width()
}

fn print_acp_tee_stdout_markdown_line(prefix: &str, rendered_payload: &str) {
    super::print_stdout_rendered_line(&format!("{prefix}{rendered_payload}"));
}

fn print_acp_tee_stdout_markdown_lines(prefix: &str, rendered_payloads: &[String]) {
    for rendered in rendered_payloads {
        print_acp_tee_stdout_markdown_line(prefix, rendered);
    }
}

fn print_stdout_acp_tee_line_with_timestamp_payload(
    ctx: &AcpTeeLineFmt<'_>,
    emit_stdout_markdown: bool,
) {
    let line_gate = TermimadStdoutGate {
        emit_stdout_markdown,
        dim_payload: ctx.dim_payload,
        allow_inline_styling: stdout_use_color(),
    };
    let prefix = acp_tee_payload_prefix(ctx);
    let prefix_len = acp_tee_payload_prefix_width(&prefix);
    let (max_payload, wrap) = super::terminal_wrap::line_wrap_for_prefix_len(
        prefix_len,
        ctx.line,
        super::terminal_wrap::stdout_allows_log_word_wrap(),
    );
    if let Some(rendered_lines) = termimad_text_lines_for_stdout(ctx.line, line_gate, max_payload) {
        print_acp_tee_stdout_markdown_lines(&prefix, &rendered_lines);
        return;
    }
    if !wrap {
        if let Some(rendered) = termimad_inline_payload_for_stdout(ctx.line, line_gate) {
            print_acp_tee_stdout_markdown_line(&prefix, &rendered);
            return;
        }
        let s = if stdout_use_color() {
            format_line_with_timestamp_acp_ansi_payload(ctx)
        } else {
            super::format_line_with_timestamp(ctx.ts, ctx.who, ctx.line)
        };
        super::print_stdout_rendered_line(&s);
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
            print_acp_tee_stdout_markdown_line(&prefix, &rendered);
            continue;
        }
        let s = if stdout_use_color() {
            format_line_with_timestamp_acp_ansi_payload(&seg_ctx)
        } else {
            super::format_line_with_timestamp(ctx.ts, ctx.who, &seg)
        };
        super::print_stdout_rendered_line(&s);
    }
}

#[cfg(test)]
mod kiss_stringify_private {
    #[test]
    fn stringify_internal_acp_tee() {
        let _ = stringify!(super::acp_tee_payload_prefix);
        let _ = stringify!(super::acp_tee_payload_prefix_width);
        let _ = stringify!(super::print_stdout_acp_tee_line_with_timestamp_payload);
        let _ = stringify!(super::print_acp_tee_stdout_markdown_line);
        let _ = stringify!(super::print_acp_tee_stdout_markdown_lines);
    }
}
