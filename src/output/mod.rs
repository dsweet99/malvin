//! Shared line-oriented formatting for stdout, stderr, and run logs.

mod acp_tee;
mod acp_tee_format;
mod acp_tee_markdown;
mod stderr_log;
mod stdout_display;
mod stdout_heartbeat;
pub(crate) mod terminal_wrap;

pub(crate) use stdout_display::{format_line_stdout, format_line_stdout_ansi};
pub(crate) use stdout_heartbeat::maybe_emit_stdout_heartbeat;

pub use stdout_display::{print_stdout_line, print_stdout_text};

pub use acp_tee::{
    AcpTeeDirection, AcpTeeLineFmt, AcpTeeStdoutEvent, TermimadStdoutGate, acp_tee_display_line,
    acp_tee_log_line, format_line_with_timestamp_acp_ansi, print_stdout_acp_tee_line,
    print_stdout_acp_tee_line_with_timestamp, print_stdout_acp_tee_line_with_timestamp_dim_plain,
    print_stdout_acp_tool_summary_tee, termimad_inline_payload_for_stdout,
    termimad_text_lines_for_stdout,
};
pub use acp_tee_format::format_line_with_timestamp_acp_ansi_payload;

#[cfg(test)]
mod acp_tee_tests;
#[cfg(test)]
mod acp_tee_termimad_tests;
#[cfg(test)]
mod format_tests;

#[cfg(test)]
use std::cell::RefCell;
use std::io::{IsTerminal, Write, stdout};
use std::path::PathBuf;
#[cfg(test)]
use std::sync::Mutex;
use std::sync::OnceLock;

pub(crate) use self::terminal_wrap::{
    stderr_line_wrap_meta, stdout_line_wrap_meta, wrap_words_bounded,
};

pub const MALVIN_WHO: &str = "malvin";
pub const WARNING_WHO: &str = "warning";
pub const ERROR_WHO: &str = "error";
pub use crate::malvin_constants::LEARNING_PLACEHOLDER;

#[cfg(test)]
thread_local! {
    static CAPTURED_STDERR_LINES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

#[cfg(test)]
pub(crate) fn push_captured_stderr_line(line: String) {
    CAPTURED_STDERR_LINES.with(|lines| lines.borrow_mut().push(line));
}

#[cfg(test)]
pub fn take_captured_stderr_lines() -> Vec<String> {
    CAPTURED_STDERR_LINES.with(|lines| std::mem::take(&mut *lines.borrow_mut()))
}

#[cfg(test)]
pub fn clear_captured_stderr_lines() {
    CAPTURED_STDERR_LINES.with(|lines| lines.borrow_mut().clear());
}

/// Announce one outgoing prompt on stdout with a single bracket line `[{bracket_label}...]`.
///
/// With full prompt logging enabled, the ACP session also prints the full rendered prompt when not
/// in raw-output mode: one timestamped stdout line per [`logical_lines`] slice, with the same `>`
/// stem as used for that mode’s stdout lines. Optional `prompts.log` mirrors that (full body or
/// name-only line);
/// for uniform prompts the trace **file** always records the full outgoing text, while `malvin do`
/// split traces keep a plain body on disk but use the `>do` stem on stdout and in `prompts.log`
/// when verbose.
pub fn print_outgoing_prompt_log(trace_who: &str, bracket_label: &str) {
    let directional_tag = format_acp_directional_tag_prefix('>', trace_who);
    let bracket_payload = format!("[{bracket_label}...]");
    print_stdout_acp_tee_line(AcpTeeDirection::ToAgent, &directional_tag, &bracket_payload);
}

/// Fixed width (Unicode scalars) for the bracket label in log lines (`[…] …`).
pub const LOG_TAG_INNER_WIDTH: usize = 15;

static LOG_USE_COLOR: OnceLock<bool> = OnceLock::new();
#[cfg(test)]
pub(crate) static STDOUT_LOG_TEST_LOCK: Mutex<()> = Mutex::new(());

const ANSI_DIM: &str = "\x1b[90m";
const ANSI_CYAN: &str = "\x1b[36m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_RED: &str = "\x1b[31m";
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
    format!("{ts} [{inner}] {line}")
}

pub(crate) fn timestamp_now_string() -> String {
    crate::time_format::timestamp_now_string()
}

#[must_use]
pub fn format_line(who: &str, line: &str) -> String {
    format_line_with_timestamp(&timestamp_now_string(), who, line)
}

pub(crate) fn who_tag_ansi(who: &str) -> &'static str {
    match who {
        WARNING_WHO => ANSI_YELLOW,
        ERROR_WHO => ANSI_RED,
        _ => ANSI_CYAN,
    }
}

/// ANSI-colored prefix for terminal output. Log files and trace files must use
/// [`format_line`] / [`format_line_with_timestamp`] instead.
#[must_use]
pub fn format_line_with_timestamp_ansi(ts: &str, who: &str, line: &str) -> String {
    let inner = format_log_tag_inner(who);
    let tag_color = who_tag_ansi(who);
    format!("{ANSI_DIM}{ts}{ANSI_RESET} {tag_color}[{inner}]{ANSI_RESET} {line}")
}

/// Call once from the binary entrypoint after parsing CLI. Disables color when `no_color` is true
/// or when `NO_COLOR` is set. Each stream applies color only when that stream is a terminal.
pub fn init_stdout_style(no_color: bool) {
    let disabled_by_env = std::env::var_os("NO_COLOR").is_some();
    let use_color = !no_color && !disabled_by_env;
    let _ = LOG_USE_COLOR.set(use_color);
    crate::output::stdout_heartbeat::spawn_wall_clock_poller_if_needed();
}

fn log_use_color() -> bool {
    *LOG_USE_COLOR.get().unwrap_or(&false)
}

pub(crate) fn stdout_use_color() -> bool {
    log_use_color() && stdout().is_terminal()
}

pub(crate) fn stderr_use_color() -> bool {
    log_use_color() && std::io::stderr().is_terminal()
}

pub fn set_stdout_log_path(path: Option<PathBuf>) {
    crate::stdout_log_path::set_stdout_log_path(path);
}

pub(crate) fn append_stdout_log_line(line: &str) {
    let Some(path) = crate::stdout_log_path::clone_stdout_log_path() else {
        return;
    };
    let line = crate::ansi_strip::strip_ansi_escapes(line);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| writeln!(f, "{line}"));
}

pub(crate) fn print_stdout_rendered_line(display: &str, log: &str) {
    maybe_emit_stdout_heartbeat();
    println!("{display}");
    append_stdout_log_line(log);
}

pub fn print_stdout_raw_line(line: &str) {
    print_stdout_rendered_line(line, line);
}

pub use stderr_log::{print_log_error, print_log_warning, print_stderr_line};

pub(crate) fn payload_after_fixed_width_bracket_tag(line: &str) -> Option<&str> {
    let after_open = line.strip_prefix('[')?;
    let (tag_end, _) = after_open.char_indices().nth(LOG_TAG_INNER_WIDTH)?;
    after_open[tag_end..].strip_prefix("] ")
}

const LOG_TIMESTAMP_LEN: usize = 19;

pub(crate) fn is_log_timestamp_token(token: &str) -> bool {
    let b = token.as_bytes();
    b.len() == LOG_TIMESTAMP_LEN
        && b[8] == b'.'
        && b[15] == b'.'
        && b[..8].iter().all(u8::is_ascii_digit)
        && b[9..15].iter().all(u8::is_ascii_digit)
        && b[16..].iter().all(u8::is_ascii_digit)
}

#[must_use]
pub fn is_command_prelude_line(line: &str) -> bool {
    const CMD: &str = "Command: ";
    if line.starts_with(CMD) {
        return true;
    }
    if let Some(payload) = payload_after_fixed_width_bracket_tag(line) {
        return payload.starts_with(CMD);
    }
    let Some((ts, rest)) = line.split_once(' ') else {
        return false;
    };
    if !is_log_timestamp_token(ts) {
        return false;
    }
    payload_after_fixed_width_bracket_tag(rest).is_some_and(|payload| payload.starts_with(CMD))
}

pub(crate) fn logical_lines(text: &str) -> impl Iterator<Item = &str> {
    text.split_inclusive('\n')
        .map(|part| part.strip_suffix('\n').unwrap_or(part))
}
