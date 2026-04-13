//! Shared line-oriented formatting for stdout, stderr, and run logs.

mod acp_tee;

pub use acp_tee::{
    AcpTeeDirection, format_line_with_timestamp_acp_ansi, print_stdout_acp_tee_line,
};

use std::io::{IsTerminal, stdout};
use std::sync::OnceLock;

use chrono::Local;

pub const MALVIN_WHO: &str = "malvin";
pub const LEARNING_PLACEHOLDER: &str = "[learning...]";

/// Fixed width (Unicode scalars) for the bracket label in log lines (`[…]: …`).
pub const LOG_TAG_INNER_WIDTH: usize = 10;

static STDOUT_USE_COLOR: OnceLock<bool> = OnceLock::new();

const ANSI_DIM: &str = "\x1b[90m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_RESET: &str = "\x1b[0m";

#[must_use]
pub fn format_log_tag_inner(label: &str) -> String {
    let mut s: String = label.chars().take(LOG_TAG_INNER_WIDTH).collect();
    while s.chars().count() < LOG_TAG_INNER_WIDTH {
        s.push(' ');
    }
    s
}

/// Outgoing (`>`) or incoming (`<`) ACP trace label before fixed-width padding (e.g. `>implement`).
#[must_use]
pub fn format_acp_directional_tag_prefix(direction: char, stem: &str) -> String {
    let mut s = String::new();
    s.push(direction);
    s.push_str(stem);
    s
}

#[must_use]
pub fn format_line_with_timestamp(ts: &str, who: &str, line: &str) -> String {
    let inner = format_log_tag_inner(who);
    format!("{ts}:[{inner}]: {line}")
}

fn timestamp_now_string() -> String {
    let now = Local::now();
    format!(
        "{}.{:03}",
        now.format("%Y%m%d.%H%M%S"),
        now.timestamp_subsec_millis()
    )
}

#[must_use]
pub fn format_line(who: &str, line: &str) -> String {
    format_line_with_timestamp(&timestamp_now_string(), who, line)
}

/// ANSI-colored prefix for terminal stdout only. Log files and trace files must use
/// [`format_line`] / [`format_line_with_timestamp`] instead.
#[must_use]
pub fn format_line_with_timestamp_ansi(ts: &str, who: &str, line: &str) -> String {
    let inner = format_log_tag_inner(who);
    format!("{ANSI_DIM}{ts}{ANSI_RESET}{ANSI_CYAN}:[{inner}]:{ANSI_RESET} {line}")
}

/// Call once from the binary entrypoint after parsing CLI. Disables color when `no_color` is true,
/// when `NO_COLOR` is set, or when stdout is not a terminal.
pub fn init_stdout_style(no_color: bool) {
    let disabled_by_env = std::env::var_os("NO_COLOR").is_some();
    let use_color = !no_color && !disabled_by_env && stdout().is_terminal();
    let _ = STDOUT_USE_COLOR.set(use_color);
}

fn stdout_use_color() -> bool {
    *STDOUT_USE_COLOR.get().unwrap_or(&false)
}

pub fn print_stdout_line(who: &str, line: &str) {
    let s = if stdout_use_color() {
        format_line_with_timestamp_ansi(&timestamp_now_string(), who, line)
    } else {
        format_line(who, line)
    };
    println!("{s}");
}

pub fn print_stderr_line(who: &str, line: &str) {
    eprintln!("{}", format_line(who, line));
}

pub fn print_stdout_text(who: &str, text: &str) {
    for line in logical_lines(text) {
        print_stdout_line(who, line);
    }
}

#[must_use]
pub fn is_command_prelude_line(line: &str) -> bool {
    line.starts_with("Command: ")
        || line
            .split_once("]: ")
            .is_some_and(|(_, payload)| payload.starts_with("Command: "))
}

pub(crate) fn logical_lines(text: &str) -> impl Iterator<Item = &str> {
    text.split_inclusive('\n')
        .map(|part| part.strip_suffix('\n').unwrap_or(part))
}

#[cfg(test)]
mod tests {
    use super::acp_tee::{AcpTeeDirection, print_stdout_acp_tee_line};
    use super::{
        LEARNING_PLACEHOLDER, LOG_TAG_INNER_WIDTH, MALVIN_WHO, format_acp_directional_tag_prefix,
        format_line, format_line_with_timestamp, format_line_with_timestamp_ansi,
        format_log_tag_inner, init_stdout_style, is_command_prelude_line, print_stderr_line,
        print_stdout_line, print_stdout_text,
    };

    #[test]
    fn formats_expected_mini_header() {
        let inner = format_log_tag_inner("kpop");
        assert_eq!(
            format_line_with_timestamp("20260413.121314.015", "kpop", "hello"),
            format!("20260413.121314.015:[{inner}]: hello")
        );
    }

    #[test]
    fn log_tag_inner_is_fixed_width() {
        assert_eq!(
            format_log_tag_inner("plan").chars().count(),
            LOG_TAG_INNER_WIDTH
        );
        let long = "a".repeat(100);
        assert_eq!(
            format_log_tag_inner(&long).chars().count(),
            LOG_TAG_INNER_WIDTH
        );
    }

    #[test]
    fn ansi_timestamp_line_keeps_payload_plain() {
        let plain = format_line_with_timestamp("20260413.121314.015", "kpop", "hello");
        assert!(!plain.contains('\x1b'));
        let ansi = format_line_with_timestamp_ansi("20260413.121314.015", "kpop", "hello");
        assert!(ansi.contains('\x1b'));
        assert!(ansi.ends_with(" hello"));
    }

    #[test]
    fn detects_prefixed_and_unprefixed_command_prelude() {
        let inner = format_log_tag_inner(MALVIN_WHO);
        assert!(is_command_prelude_line("Command: malvin code @plan.md"));
        assert!(is_command_prelude_line(&format!(
            "20260413.121314.015:[{inner}]: Command: malvin code @plan.md"
        )));
        assert!(!is_command_prelude_line(
            "20260413.121314.015:[kpop]: not a command line"
        ));
    }

    #[test]
    fn exported_constants_match_public_contract() {
        assert_eq!(MALVIN_WHO, "malvin");
        assert_eq!(LEARNING_PLACEHOLDER, "[learning...]");
    }

    #[test]
    fn smoke_print_and_format_paths_cover_helpers() {
        assert_eq!(format_acp_directional_tag_prefix('>', "x"), ">x");
        let _ = format_line("who", "body");
        init_stdout_style(true);
        print_stdout_line("u", "one");
        print_stdout_acp_tee_line(AcpTeeDirection::FromAgent, "<w", "two");
        print_stderr_line("e", "err");
        print_stdout_text("t", "a\nb");
        let mut it = super::logical_lines("x\ny");
        assert_eq!(it.next(), Some("x"));
        assert_eq!(it.next(), Some("y"));
    }

    #[test]
    fn kiss_stringify_output_symbols() {
        let _ = stringify!(super::format_log_tag_inner);
        let _ = stringify!(super::format_acp_directional_tag_prefix);
        let _ = stringify!(super::format_line_with_timestamp);
        let _ = stringify!(super::timestamp_now_string);
        let _ = stringify!(super::format_line);
        let _ = stringify!(super::format_line_with_timestamp_ansi);
        let _ = stringify!(super::init_stdout_style);
        let _ = stringify!(super::stdout_use_color);
        let _ = stringify!(super::print_stdout_line);
        let _ = stringify!(super::print_stderr_line);
        let _ = stringify!(super::print_stdout_text);
        let _ = stringify!(super::is_command_prelude_line);
        let _ = stringify!(super::logical_lines);
    }
}
