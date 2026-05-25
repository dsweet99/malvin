use std::cell::RefCell;

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
    #[cfg(test)]
    if CAPTURE_STDOUT.with(|flag| *flag.borrow()) {
        CAPTURED_STDOUT_LINES.with(|lines| lines.borrow_mut().push(display.to_string()));
        return;
    }
    println!("{display}");
}

#[cfg(test)]
mod tests {
    use super::{enable_stdout_capture, print_stdout_display_line, take_captured_stdout};

    #[test]
    fn capture_routes_display_without_real_stdout() {
        enable_stdout_capture();
        print_stdout_display_line("[malvin.........] cap");
        assert_eq!(take_captured_stdout(), "[malvin.........] cap");
    }
}
