pub(crate) fn emit_stdout_rendered_immediate(display: &str, log: &str) {
    if !display.is_empty() {
        super::print_stdout_display_line(display);
    }
    if !log.is_empty() {
        super::append_stdout_log_line(log);
    }
}
