//! Shared line-oriented formatting for stdout, stderr, and run logs.

mod acp_tee;
mod acp_tee_markdown;
mod stderr_log;
pub(crate) mod terminal_wrap;

pub use acp_tee::{
    AcpTeeDirection, AcpTeeStdoutEvent, TermimadStdoutGate, format_line_with_timestamp_acp_ansi,
    print_stdout_acp_tee_line, print_stdout_acp_tee_line_with_timestamp,
    print_stdout_acp_tee_line_with_timestamp_dim_plain, termimad_inline_payload_for_stdout,
    termimad_text_lines_for_stdout,
};

#[cfg(test)]
mod acp_tee_tests;
#[cfg(test)]
mod format_tests;

use std::io::{IsTerminal, Write, stdout};
use std::path::PathBuf;
use std::sync::OnceLock;
#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
use std::sync::Mutex;

pub(crate) use self::terminal_wrap::{stderr_line_wrap_meta, stdout_line_wrap_meta, wrap_words_bounded};

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

/// Announce one outgoing prompt on stdout with a single bracket line `[{label}...]`.
///
/// With full prompt logging enabled, the ACP session also prints the full rendered prompt when not
/// in raw-output mode: one timestamped stdout line per [`logical_lines`] slice, with the same `>`
/// stem as used for that mode’s stdout lines. Optional `prompts.log` mirrors that (full body or
/// name-only line);
/// for uniform prompts the trace **file** always records the full outgoing text, while `malvin do`
/// split traces keep a plain body on disk but use the `>do` stem on stdout and in `prompts.log`
/// when verbose.
pub fn print_outgoing_prompt_log(label: &str) {
    let directional_tag = format_acp_directional_tag_prefix('>', label);
    let bracket_payload = format!("[{label}...]");
    print_stdout_acp_tee_line(AcpTeeDirection::ToAgent, &directional_tag, &bracket_payload);
}

/// Fixed width (Unicode scalars) for the bracket label in log lines (`[…]: …`).
pub const LOG_TAG_INNER_WIDTH: usize = 10;

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
    format!("{ts}:[{inner}]: {line}")
}

pub(crate) fn timestamp_now_string() -> String {
    crate::time_format::timestamp_now_string()
}

#[must_use]
pub fn format_line(who: &str, line: &str) -> String {
    format_line_with_timestamp(&timestamp_now_string(), who, line)
}

fn who_tag_ansi(who: &str) -> &'static str {
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
    format!("{ANSI_DIM}{ts}{ANSI_RESET}{tag_color}:[{inner}]:{ANSI_RESET} {line}")
}

/// Call once from the binary entrypoint after parsing CLI. Disables color when `no_color` is true
/// or when `NO_COLOR` is set. Each stream applies color only when that stream is a terminal.
pub fn init_stdout_style(no_color: bool) {
    let disabled_by_env = std::env::var_os("NO_COLOR").is_some();
    let use_color = !no_color && !disabled_by_env;
    let _ = LOG_USE_COLOR.set(use_color);
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

pub(crate) fn print_stdout_rendered_line(line: &str) {
    println!("{line}");
    append_stdout_log_line(line);
}

pub fn print_stdout_raw_line(line: &str) {
    print_stdout_rendered_line(line);
}

pub fn print_stdout_line(who: &str, line: &str) {
    let ts = timestamp_now_string();
    let (max_payload, wrap) = stdout_line_wrap_meta(&ts, who, line);
    if !wrap {
        let s = if stdout_use_color() {
            format_line_with_timestamp_ansi(&ts, who, line)
        } else {
            format_line_with_timestamp(&ts, who, line)
        };
        print_stdout_rendered_line(&s);
        return;
    }
    for seg in wrap_words_bounded(max_payload, line) {
        let s = if stdout_use_color() {
            format_line_with_timestamp_ansi(&ts, who, &seg)
        } else {
            format_line_with_timestamp(&ts, who, &seg)
        };
        print_stdout_rendered_line(&s);
    }
}

pub use stderr_log::{print_log_error, print_log_warning, print_stderr_line};

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
