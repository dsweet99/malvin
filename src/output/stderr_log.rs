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
    for para in line.split('\n') {
        let ts = timestamp_now_string();
        let (max_payload, wrap) = stderr_line_wrap_meta(&ts, who, para);
        if !wrap {
            emit_stderr_log_line(&ts, who, para);
            continue;
        }
        for seg in wrap_words_bounded(max_payload, para) {
            emit_stderr_log_line(&ts, who, &seg);
        }
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
    fn emit_stderr_log_line_emit_stderr_log_lines_capture_error() {
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

    #[test]
    #[allow(unsafe_code)]
    fn multiline_log_error_tags_every_physical_line() {
        let prev_cols = std::env::var("COLUMNS").ok();
        unsafe {
            std::env::set_var("COLUMNS", "500");
        }
        crate::output::clear_captured_stderr_lines();
        super::print_log_error("first\nsecond\nthird");
        let lines = crate::output::take_captured_stderr_lines();
        unsafe {
            match prev_cols {
                Some(v) => std::env::set_var("COLUMNS", v),
                None => std::env::remove_var("COLUMNS"),
            }
        }
        assert_eq!(lines.len(), 3, "expected one captured line per paragraph: {lines:?}");
        for line in &lines {
            assert!(
                line.contains("[error"),
                "each physical line must carry the error tag: {line:?}"
            );
        }
        assert!(lines[0].contains("first"));
        assert!(lines[1].contains("second"));
        assert!(lines[2].contains("third"));
    }
}
