//! ACP trace tee: distinct ANSI colors for outbound (`>`) vs inbound (`<`) lines on stdout.

use super::{format_line, format_log_tag_inner, timestamp_now_string, stdout_use_color};
use super::{ANSI_DIM, ANSI_RESET};

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
    let inner = format_log_tag_inner(who);
    let bracket = match direction {
        AcpTeeDirection::ToAgent => ANSI_BRIGHT_GREEN,
        AcpTeeDirection::FromAgent => ANSI_BRIGHT_MAGENTA,
    };
    format!("{ANSI_DIM}{ts}{ANSI_RESET}{bracket}:[{inner}]:{ANSI_RESET} {line}")
}

/// Stdout tee for ACP trace lines: when color is enabled, outbound (`>`) vs inbound (`<`) use
/// different ANSI colors for the `[who]:` prefix; payload text stays unstyled.
pub fn print_stdout_acp_tee_line(direction: AcpTeeDirection, who: &str, line: &str) {
    let s = if stdout_use_color() {
        format_line_with_timestamp_acp_ansi(&timestamp_now_string(), direction, who, line)
    } else {
        format_line(who, line)
    };
    println!("{s}");
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
    fn kiss_stringify_acp_tee() {
        let _ = stringify!(AcpTeeDirection);
        let _ = stringify!(AcpTeeDirection::ToAgent);
        let _ = stringify!(AcpTeeDirection::FromAgent);
        let _ = stringify!(super::format_line_with_timestamp_acp_ansi);
        let _ = stringify!(super::print_stdout_acp_tee_line);
    }
}
