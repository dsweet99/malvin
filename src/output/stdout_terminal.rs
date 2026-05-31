use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};

static STDOUT_SUPPRESSED: AtomicBool = AtomicBool::new(false);

/// When true, [`print_stdout_display_line`] is a no-op (stderr and log files unchanged).
pub fn set_stdout_suppressed(suppress: bool) {
    STDOUT_SUPPRESSED.store(suppress, Ordering::Relaxed);
}

#[must_use]
pub fn stdout_suppressed() -> bool {
    STDOUT_SUPPRESSED.load(Ordering::Relaxed)
}

thread_local! {
    static CAPTURE_STDOUT: RefCell<bool> = const { RefCell::new(false) };
    static CAPTURED_STDOUT_LINES: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

#[cfg(test)]
pub(crate) fn enable_stdout_capture() {
    CAPTURE_STDOUT.with(|flag| *flag.borrow_mut() = true);
    CAPTURED_STDOUT_LINES.with(|lines| lines.borrow_mut().clear());
}

#[cfg(test)]
pub(crate) fn take_captured_stdout() -> String {
    CAPTURE_STDOUT.with(|flag| *flag.borrow_mut() = false);
    CAPTURED_STDOUT_LINES.with(|lines| lines.borrow_mut().join("\n"))
}

pub(crate) fn print_stdout_display_line(display: &str) {
    if stdout_suppressed() {
        return;
    }
    #[cfg(test)]
    if CAPTURE_STDOUT.with(|flag| *flag.borrow()) {
        CAPTURED_STDOUT_LINES.with(|lines| lines.borrow_mut().push(display.to_string()));
        return;
    }
    println!("{display}");
}

#[cfg(test)]
mod tests {
    use super::{
        enable_stdout_capture, print_stdout_display_line, set_stdout_suppressed,
        take_captured_stdout,
    };

    #[test]
    fn capture_routes_display_without_real_stdout() {
        enable_stdout_capture();
        print_stdout_display_line("malvin.| cap");
        assert_eq!(take_captured_stdout(), "malvin.| cap");
    }

    #[test]
    fn background_mode_suppresses_stdout_display() {
        set_stdout_suppressed(true);
        assert!(super::stdout_suppressed());
        enable_stdout_capture();
        print_stdout_display_line("malvin.| hidden");
        assert!(take_captured_stdout().is_empty());
        set_stdout_suppressed(false);
        assert!(!super::stdout_suppressed());
    }
}
