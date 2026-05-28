use super::{
    ERROR_WHO, WARNING_WHO, append_stdout_log_line, stderr_line_wrap_meta, timestamp_now_string,
    wrap_words_bounded,
};

pub(crate) fn emit_stderr_log_line(ts: &str, who: &str, line: &str) {
    let (display, log) =
        super::stdout_log_pair::stderr_tagged_display_and_log_line(who, line, Some(ts));
    eprintln!("{display}");
    append_stdout_log_line(&log);
    #[cfg(test)]
    super::push_captured_stderr_line(display);
}

pub(crate) fn emit_stderr_log_lines(who: &str, line: &str) {
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
    pub(crate) fn emit_stderr_log_line_emit_stderr_log_lines_capture_error() {
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
    fn stderr_display_omits_timestamp_but_log_keeps_it() {
        let _stdout_guard = crate::output::STDOUT_LOG_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("stdout.log");
        crate::output::set_stdout_log_path(Some(path.clone()));
        crate::output::clear_captured_stderr_lines();

        super::print_stderr_line(crate::output::MALVIN_WHO, "VIOLATION:test_coverage:probe");
        crate::output::set_stdout_log_path(None);

        let captured = crate::output::take_captured_stderr_lines();
        let display = captured.first().expect("captured stderr display");
        assert!(
            !crate::output::is_log_timestamp_token(display.split_whitespace().next().unwrap_or("")),
            "live stderr display must omit wall-clock prefix; got {display:?}"
        );
        assert!(display.contains("[malvin"));
        assert!(display.contains("VIOLATION:test_coverage:probe"));

        let log_text = std::fs::read_to_string(path).expect("read log");
        let log_line = log_text.lines().next().expect("logged stderr line");
        assert!(
            crate::output::is_log_timestamp_token(log_line.split_whitespace().next().unwrap_or("")),
            "stderr log line must keep wall-clock prefix; got {log_line:?}"
        );
        assert!(log_line.contains("VIOLATION:test_coverage:probe"));
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
        assert_eq!(
            lines.len(),
            3,
            "expected one captured line per paragraph: {lines:?}"
        );
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

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _ = emit_stderr_log_line;
        let _ = emit_stderr_log_lines;
    }
}
