use super::{
    ERROR_WHO, WARNING_WHO, append_stdout_log_line, format_line_with_timestamp,
    format_line_with_timestamp_ansi, stderr_line_wrap_meta, stderr_use_color, timestamp_now_string,
    wrap_words_bounded,
};

fn emit_stderr_log_line(ts: &str, who: &str, line: &str) {
    let formatted = if stderr_use_color() {
        format_line_with_timestamp_ansi(ts, who, line)
    } else {
        format_line_with_timestamp(ts, who, line)
    };
    eprintln!("{formatted}");
    append_stdout_log_line(&formatted);
    #[cfg(test)]
    super::push_captured_stderr_line(formatted);
}

fn emit_stderr_log_lines(who: &str, line: &str) {
    let ts = timestamp_now_string();
    let (max_payload, wrap) = stderr_line_wrap_meta(&ts, who, line);
    if !wrap {
        emit_stderr_log_line(&ts, who, line);
        return;
    }
    for seg in wrap_words_bounded(max_payload, line) {
        emit_stderr_log_line(&ts, who, &seg);
    }
}

pub fn print_stderr_line(who: &str, line: &str) {
    emit_stderr_log_lines(who, line);
}

pub fn print_log_warning(line: &str) {
    emit_stderr_log_lines(WARNING_WHO, line);
}

pub fn print_log_error(line: &str) {
    emit_stderr_log_lines(ERROR_WHO, line);
}

#[cfg(test)]
mod stderr_log_tests {
    #[test]
    fn emit_stderr_log_lines_captured_in_tests() {
        crate::output::clear_captured_stderr_lines();
        super::print_log_error("stderr-log-smoke");
        let lines = crate::output::take_captured_stderr_lines();
        assert!(lines.iter().any(|l| l.contains("stderr-log-smoke")));
    }

    #[test]
    fn emit_stderr_log_lines_captures_warning() {
        crate::output::clear_captured_stderr_lines();
        super::print_log_warning("warn-smoke");
        let lines = crate::output::take_captured_stderr_lines();
        assert!(lines.iter().any(|l| l.contains("warn-smoke")));
    }
}
