//! ACP trace tee: distinct ANSI colors for outbound (`>`) vs inbound (`<`) lines on stdout.

use super::{ANSI_DIM, ANSI_RESET};
use super::{format_log_tag_inner, stdout_use_color, timestamp_now_string};

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
fn format_line_with_timestamp_acp_ansi_payload(ctx: &AcpTeeLineFmt<'_>) -> String {
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
/// different ANSI colors for the `[who]:` prefix; payload text stays unstyled.
pub fn print_stdout_acp_tee_line(direction: AcpTeeDirection, who: &str, line: &str) {
    let ts = timestamp_now_string();
    print_stdout_acp_tee_line_with_timestamp(direction, who, line, &ts);
}

/// Same as [`print_stdout_acp_tee_line`], but uses `ts` for the line prefix (shared with disk trace).
pub fn print_stdout_acp_tee_line_with_timestamp(
    direction: AcpTeeDirection,
    who: &str,
    line: &str,
    ts: &str,
) {
    print_stdout_acp_tee_line_with_timestamp_payload(&AcpTeeLineFmt {
        ts,
        direction,
        who,
        line,
        dim_payload: false,
    });
}

/// Same as [`print_stdout_acp_tee_line_with_timestamp`], but dims the payload on colorized stdout.
pub fn print_stdout_acp_tee_line_with_timestamp_dim_payload(
    direction: AcpTeeDirection,
    who: &str,
    line: &str,
    ts: &str,
) {
    print_stdout_acp_tee_line_with_timestamp_payload(&AcpTeeLineFmt {
        ts,
        direction,
        who,
        line,
        dim_payload: true,
    });
}

fn print_stdout_acp_tee_line_with_timestamp_payload(ctx: &AcpTeeLineFmt<'_>) {
    let (max_payload, wrap) =
        super::terminal_wrap::stdout_line_wrap_meta(ctx.ts, ctx.who, ctx.line);
    if !wrap {
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
        let s = if stdout_use_color() {
            format_line_with_timestamp_acp_ansi_payload(&seg_ctx)
        } else {
            super::format_line_with_timestamp(ctx.ts, ctx.who, &seg)
        };
        println!("{s}");
    }
}

#[cfg(test)]
mod tests {
    use super::{AcpTeeDirection, format_line_with_timestamp_acp_ansi};

    #[test]
    fn ansi_acp_tee_directions_use_distinct_bracket_colors() {
        let to_line = format_line_with_timestamp_acp_ansi(
            "20260413.121314.015",
            AcpTeeDirection::ToAgent,
            ">stem",
            "out",
        );
        let from_line = format_line_with_timestamp_acp_ansi(
            "20260413.121314.015",
            AcpTeeDirection::FromAgent,
            "<stem",
            "in",
        );
        assert!(to_line.contains('\x1b'));
        assert!(from_line.contains('\x1b'));
        assert_ne!(to_line, from_line);
        assert!(to_line.ends_with(" out"));
        assert!(from_line.ends_with(" in"));
    }

    #[test]
    fn ansi_acp_tee_can_dim_payload_text() {
        let line = super::format_line_with_timestamp_acp_ansi_payload(&super::AcpTeeLineFmt {
            ts: "20260413.121314.015",
            direction: AcpTeeDirection::FromAgent,
            who: "<stem",
            line: "[thinking]",
            dim_payload: true,
        });
        assert!(line.contains("\x1b[90m[thinking]\x1b[0m"));
    }

    #[test]
    fn kiss_stringify_acp_tee() {
        let _ = stringify!(AcpTeeDirection);
        let _ = stringify!(AcpTeeDirection::ToAgent);
        let _ = stringify!(AcpTeeDirection::FromAgent);
        let _ = stringify!(super::AcpTeeLineFmt);
        let _ = stringify!(super::format_line_with_timestamp_acp_ansi);
        let _ = stringify!(super::format_line_with_timestamp_acp_ansi_payload);
        let _ = stringify!(super::print_stdout_acp_tee_line);
        let _ = stringify!(super::print_stdout_acp_tee_line_with_timestamp);
        let _ = stringify!(super::print_stdout_acp_tee_line_with_timestamp_dim_payload);
    }
}
