//! ACP trace tee: distinct ANSI colors for outbound (`>`) vs inbound (`<`) lines on stdout.

pub use super::acp_tee_markdown::{TermimadStdoutGate, termimad_inline_payload_for_stdout};
use super::{ANSI_DIM, ANSI_RESET};
use super::{format_log_tag_inner, stdout_use_color, timestamp_now_string};

use crate::ansi_strip::strip_ansi_escapes;

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

/// Same as [`print_stdout_acp_tee_line_with_timestamp`], but dims the payload on colorized stdout.
pub fn print_stdout_acp_tee_line_with_timestamp_dim_payload(
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

fn acp_tee_log_prefix_len(ctx: &AcpTeeLineFmt<'_>) -> usize {
    let s = if super::stdout_use_color() {
        format_line_with_timestamp_acp_ansi_payload(&AcpTeeLineFmt {
            ts: ctx.ts,
            direction: ctx.direction,
            who: ctx.who,
            line: "",
            dim_payload: ctx.dim_payload,
        })
    } else {
        super::format_line_with_timestamp(ctx.ts, ctx.who, "")
    };
    strip_ansi_escapes(&s).chars().count()
}

fn print_acp_tee_stdout_markdown_line(ctx: &AcpTeeLineFmt<'_>, rendered_payload: &str) {
    let prefix = format_line_with_timestamp_acp_ansi_payload(&AcpTeeLineFmt {
        ts: ctx.ts,
        direction: ctx.direction,
        who: ctx.who,
        line: "",
        dim_payload: ctx.dim_payload,
    });
    println!("{prefix}{rendered_payload}");
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
    let prefix_len = acp_tee_log_prefix_len(ctx);
    let (max_payload, wrap) = super::terminal_wrap::line_wrap_for_prefix_len(
        prefix_len,
        ctx.line,
        super::terminal_wrap::stdout_allows_log_word_wrap(),
    );
    if !wrap {
        if let Some(rendered) =
            termimad_inline_payload_for_stdout(ctx.line, &line_gate)
        {
            print_acp_tee_stdout_markdown_line(ctx, &rendered);
            return;
        }
        let s = if stdout_use_color() {
            format_line_with_timestamp_acp_ansi_payload(ctx)
        } else {
            super::format_line_with_timestamp(ctx.ts, ctx.who, ctx.line)
        };
        println!("{s}");
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
        if let Some(rendered) = termimad_inline_payload_for_stdout(&seg, &line_gate) {
            print_acp_tee_stdout_markdown_line(&seg_ctx, &rendered);
            continue;
        }
        let s = if stdout_use_color() {
            format_line_with_timestamp_acp_ansi_payload(&seg_ctx)
        } else {
            super::format_line_with_timestamp(ctx.ts, ctx.who, &seg)
        };
        println!("{s}");
    }
}

#[cfg(test)]
mod kiss_stringify_private {
    #[test]
    fn stringify_internal_acp_tee() {
        let _ = stringify!(super::acp_tee_log_prefix_len);
        let _ = stringify!(super::print_stdout_acp_tee_line_with_timestamp_payload);
        let _ = stringify!(super::print_acp_tee_stdout_markdown_line);
    }
}
