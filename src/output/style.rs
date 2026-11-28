#[cfg(test)]
use std::cell::RefCell;
use std::io::{IsTerminal, Write, stdout};
#[cfg(test)]
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::terminal_palette::{ANSI_DIM, ansi_error, ansi_warning, ansi_who_tag};

pub const MALVIN_WHO: &str = super::who_tag::WHO_O;
pub const WARNING_WHO: &str = "w";
pub const ERROR_WHO: &str = "e";

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

static LOG_USE_COLOR: AtomicBool = AtomicBool::new(false);
#[cfg(test)]
pub(crate) static STDOUT_LOG_TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn set_log_use_color(use_color: bool) {
    LOG_USE_COLOR.store(use_color, Ordering::Relaxed);
}

pub(crate) fn who_tag_ansi(who: &str) -> &'static str {
    match who {
        WARNING_WHO => ansi_warning(),
        ERROR_WHO => ansi_error(),
        super::who_tag::WHO_B => ANSI_DIM,
        _ => ansi_who_tag(),
    }
}

pub(crate) fn log_use_color() -> bool {
    LOG_USE_COLOR.load(Ordering::Relaxed)
}

pub(crate) fn stdout_use_color() -> bool {
    log_use_color() && stdout().is_terminal()
}

pub(crate) fn stderr_use_color() -> bool {
    log_use_color() && std::io::stderr().is_terminal()
}

pub(crate) fn append_stdout_log_line(line: &str) {
    let Some(path) = crate::workspace::stdout_log_path::clone_stdout_log_path() else {
        return;
    };
    let line = crate::ansi_strip::strip_ansi_escapes(line);
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| writeln!(f, "{line}"));
}
