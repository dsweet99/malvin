pub fn capture_stderr_output(f: impl FnOnce()) -> String {
    crate::output::clear_captured_stderr_lines();
    f();
    crate::output::take_captured_stderr_lines().join("")
}
