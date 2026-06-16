//! Shared line-oriented formatting for stdout, stderr, and run logs.

pub(crate) mod acp_tee;
mod acp_tee_markdown;
pub(crate) mod stderr_log;
mod stdout_defer;
mod stdout_display;
mod stdout_heartbeat;
mod stdout_render;
mod stdout_terminal;
mod test_modules;
mod who_tag;
pub(crate) mod stdout_log_pair;
pub(crate) mod stdout_tee_env;
pub(crate) mod terminal_wrap;

pub(crate) use stdout_defer::register_defer_stdout_hooks;
#[allow(unused_imports)]
pub(crate) use stdout_defer::{try_defer_heartbeat, try_defer_tagged_stdout};
pub(crate) use stdout_render::{
    flush_stdout_rendered_line, publish_heartbeat_live_terminal, write_heartbeat_log_line,
};
#[cfg(test)]
pub(crate) use stdout_render::emit_stdout_rendered_immediate;
pub(crate) use stdout_heartbeat::{
    heartbeat_rendered_if_due, log_contains_heartbeat,
};

pub(crate) use stdout_display::{
    format_heartbeat_stdout_ansi, format_line_stdout, format_line_stdout_ansi, logical_lines,
};

#[cfg(test)]
pub(crate) use stdout_heartbeat::{
    heartbeat_log_offset, poll_wall_clock_heartbeat_if_due, reset_stdout_heartbeat_for_test,
    test_set_last_heartbeat_elapsed, HEARTBEAT_TEST_LOCK,
};

pub use stdout_display::{
    print_stdout_line, print_stdout_raw_line, print_stdout_raw_line_with_ts, print_stdout_text,
};
pub use stdout_terminal::{set_stdout_suppressed, stdout_suppressed};

pub(crate) use acp_tee::{
    flush_stdout_acp_tee_line_with_timestamp, flush_stdout_acp_tool_summary_tee,
};
pub(crate) use stdout_display::flush_stdout_raw_line_with_ts;
pub use acp_tee::{
    AcpTeeDirection, AcpTeeLineFmt, AcpTeeStdoutEvent, TermimadStdoutGate, acp_tee_display_line,
    acp_tee_log_line, format_line_acp_ansi_payload, print_stdout_acp_tee_line,
    print_stdout_acp_tee_line_with_timestamp, print_stdout_acp_tee_line_with_timestamp_dim_plain,
    print_stdout_acp_tool_summary_tee, termimad_inline_payload_for_stdout,
    termimad_text_lines_for_stdout,
};

#[cfg(test)]
mod acp_tee_termimad_tests;
#[cfg(test)]
mod acp_tee_tests;
#[cfg(test)]
mod format_tests;
#[cfg(test)]
#[path = "format_tests_b.rs"]
mod format_tests_b;
#[cfg(test)]
mod stdout_log_tests;
#[cfg(test)]
#[path = "output_kiss_cov_tests.rs"]
mod output_kiss_cov_tests;

#[cfg(test)]
use std::cell::RefCell;
use std::io::{IsTerminal, Write, stdout};
#[cfg(test)]
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

pub(crate) use self::terminal_wrap::{
    stderr_line_wrap_meta, stdout_line_wrap_meta, wrap_words_bounded,
};

pub const MALVIN_WHO: &str = who_tag::WHO_O;
pub const WARNING_WHO: &str = "w";
pub const ERROR_WHO: &str = "e";

pub(crate) use stdout_terminal::print_stdout_display_line;
#[cfg(test)]
pub(crate) use stdout_terminal::{enable_stdout_capture, take_captured_stdout};

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

/// Record one outgoing prompt summary in stdout.log only (not on the live terminal).
pub fn print_outgoing_prompt_log(_trace_who: &str, bracket_label: &str) {
    let bracket_payload = format!("[{bracket_label}...]");
    let ts = timestamp_now_string();
    append_stdout_log_line(&stdout_log_pair::tagged_log_line(
        &ts,
        who_tag::WHO_U,
        &bracket_payload,
    ));
}

pub fn append_outgoing_prompt_log_lines(body: &str) {
    let ts = timestamp_now_string();
    for line in stdout_display::logical_lines(body) {
        append_stdout_log_line(&stdout_log_pair::tagged_log_line(
            &ts,
            who_tag::WHO_U,
            line,
        ));
    }
}

pub use who_tag::{
    format_acp_directional_tag_prefix, format_log_tag_inner, format_who_tag_delim,
    format_who_tag_prefix, is_command_prelude_line, LOG_TAG_INNER_WIDTH, WHO_B, WHO_H, WHO_M,
    WHO_O, WHO_T, WHO_U,
};

#[allow(unused_imports)]
pub(crate) use who_tag::{
    is_log_timestamp_token, payload_after_fixed_width_bracket_tag,
    payload_after_fixed_width_who_tag,
};

static LOG_USE_COLOR: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
pub(crate) static STDOUT_LOG_TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) use crate::terminal_palette::{
    ansi_tool_amber, ansi_tool_coral, ansi_tool_navy, ANSI_DIM, ANSI_RESET,
};

#[must_use]
pub fn format_line_with_timestamp(ts: &str, who: &str, line: &str) -> String {
    stdout_log_pair::tagged_log_line(ts, who, line)
}

pub(crate) use crate::time_format::timestamp_now_string;

#[must_use]
pub fn format_line(who: &str, line: &str) -> String {
    stdout_log_pair::stdout_tagged_display_and_log_line(who, line, None).1
}

pub(crate) fn who_tag_ansi(who: &str) -> &'static str {
    match who {
        WARNING_WHO => ansi_tool_amber(),
        ERROR_WHO => ansi_tool_coral(),
        who_tag::WHO_B => ANSI_DIM,
        _ => ansi_tool_navy(),
    }
}

/// ANSI-colored prefix for terminal output. Log files and trace files must use
/// [`format_line`] / [`format_line_with_timestamp`] instead.
#[must_use]
pub fn format_line_with_timestamp_ansi(ts: &str, who: &str, line: &str) -> String {
    stdout_log_pair::tagged_display_line_with_timestamp_ansi(ts, who, line)
}

/// Call once from the binary entrypoint after parsing CLI. Disables color when `no_color` is true
/// or when `NO_COLOR` is set. Each stream applies color only when that stream is a terminal.
pub fn init_stdout_style(no_color: bool) {
    let disabled_by_env = std::env::var_os("NO_COLOR").is_some();
    let use_color = !no_color && !disabled_by_env;
    LOG_USE_COLOR.store(use_color, Ordering::Relaxed);
    crate::output::stdout_heartbeat::spawn_wall_clock_poller_if_needed();
}

pub(crate) fn log_use_color() -> bool {
    LOG_USE_COLOR.load(Ordering::Relaxed)
}

pub(crate) fn stdout_use_color() -> bool {
    log_use_color() && stdout().is_terminal()
}

pub use stdout_tee_env::{
    agent_stdout_tee_enabled, force_stdout_tee_from_env, stdout_is_interactive,
};

pub(crate) fn stderr_use_color() -> bool {
    log_use_color() && std::io::stderr().is_terminal()
}

pub use crate::stdout_log_path::set_stdout_log_path;
pub(crate) use stdout_log_pair::{
    stdout_heartbeat_display_and_log_line, stdout_tagged_display_and_log_line,
};

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

pub use stderr_log::{print_log_error, print_log_warning, print_stderr_line};

#[cfg(test)]
pub(crate) use test_modules::{
    assert_acp_tool_summary_dim_preserves_bracket, assert_tool_payload_uses_verb_styling,
};
