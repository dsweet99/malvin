use std::cell::RefCell;

use super::stdout_mode::{StdoutMode, stdout_mode};

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
    if stdout_mode() == StdoutMode::Normal {
        emit_stdout_display_line_raw(display);
    }
}

pub(crate) fn emit_do_dm_body_line(line: &str) {
    if stdout_mode() != StdoutMode::Silent {
        emit_stdout_display_line_raw(line);
    }
}

fn emit_stdout_display_line_raw(display: &str) {
    #[cfg(test)]
    if CAPTURE_STDOUT.with(|flag| *flag.borrow()) {
        CAPTURED_STDOUT_LINES.with(|lines| lines.borrow_mut().push(display.to_string()));
        return;
    }
    #[cfg(test)]
    println!("{display}");
    #[cfg(not(test))]
    write_display_line(&mut std::io::stdout().lock(), display);
}

pub(crate) fn write_display_line(out: &mut impl std::io::Write, display: &str) {
    if let Err(e) = writeln!(out, "{display}")
        && e.kind() != std::io::ErrorKind::BrokenPipe
    {
        panic!("failed printing display line: {e}");
    }
}

#[cfg(test)]
mod tests {
    use super::{enable_stdout_capture, print_stdout_display_line, take_captured_stdout};

    #[test]
    fn capture_routes_display_without_real_stdout() {
        enable_stdout_capture();
        print_stdout_display_line("malvin.| cap");
        assert_eq!(take_captured_stdout(), "malvin.| cap");
    }

    #[test]
    fn silent_mode_discards_display_and_dm_lines() {
        let _dm = crate::output::DO_DM_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _silent = crate::output::enter_stdout_mode(crate::output::StdoutMode::Silent);
        enable_stdout_capture();
        print_stdout_display_line("malvin.| hidden");
        super::emit_do_dm_body_line("dm hidden");
        assert!(take_captured_stdout().is_empty());
    }

    struct FailingWriter(std::io::ErrorKind);

    impl std::io::Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(self.0.into())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn closed_pipe_is_ignored_but_other_write_errors_panic() {
        super::write_display_line(&mut FailingWriter(std::io::ErrorKind::BrokenPipe), "x");
        let other = std::panic::catch_unwind(|| {
            super::write_display_line(&mut FailingWriter(std::io::ErrorKind::StorageFull), "x");
        });
        assert!(other.is_err());
    }
}
