use super::{
    format_line_with_timestamp, format_log_tag_inner, stdout_use_color, timestamp_now_string,
    wrap_words_bounded, ANSI_RESET,
};
use super::stdout_line_wrap_meta;

pub(crate) use super::who_tag_ansi;

#[must_use]
pub fn format_line_stdout(who: &str, line: &str) -> String {
    let inner = format_log_tag_inner(who);
    format!("[{inner}] {line}")
}

#[must_use]
pub fn format_line_stdout_ansi(who: &str, line: &str) -> String {
    let inner = format_log_tag_inner(who);
    let tag_color = who_tag_ansi(who);
    format!("{tag_color}[{inner}]{ANSI_RESET} {line}")
}

pub(crate) fn stdout_display_and_log(ts: &str, who: &str, line: &str) -> (String, String) {
    let log = format_line_with_timestamp(ts, who, line);
    let display = if stdout_use_color() {
        format_line_stdout_ansi(who, line)
    } else {
        format_line_stdout(who, line)
    };
    (display, log)
}

pub fn print_stdout_line(who: &str, line: &str) {
    let ts = timestamp_now_string();
    let (max_payload, wrap) = stdout_line_wrap_meta(who, line);
    if !wrap {
        let (display, log) = stdout_display_and_log(&ts, who, line);
        super::print_stdout_rendered_line(&display, &log);
        return;
    }
    for seg in wrap_words_bounded(max_payload, line) {
        let (display, log) = stdout_display_and_log(&ts, who, &seg);
        super::print_stdout_rendered_line(&display, &log);
    }
}

pub fn print_stdout_text(who: &str, text: &str) {
    for line in super::logical_lines(text) {
        print_stdout_line(who, line);
    }
}

#[cfg(test)]
mod tests {
    use super::{format_line_stdout, format_line_stdout_ansi};
    use crate::output::{LOG_TAG_INNER_WIDTH, format_log_tag_inner};

    #[test]
    fn stdout_line_omits_timestamp_prefix() {
        let inner = format_log_tag_inner("kpop");
        assert_eq!(format_line_stdout("kpop", "hello"), format!("[{inner}] hello"));
        assert!(!format_line_stdout("kpop", "hello").starts_with("20"));
    }

    #[test]
    fn stdout_ansi_line_omits_timestamp_prefix() {
        let plain = format_line_stdout("kpop", "hello");
        let ansi = format_line_stdout_ansi("kpop", "hello");
        assert!(ansi.contains('\x1b'));
        assert!(ansi.ends_with(" hello"));
        assert!(!plain.contains('\x1b'));
    }

    #[test]
    fn log_tag_inner_width_is_fifteen() {
        assert_eq!(LOG_TAG_INNER_WIDTH, 15);
        assert_eq!(format_log_tag_inner("x").chars().count(), 15);
    }
}
