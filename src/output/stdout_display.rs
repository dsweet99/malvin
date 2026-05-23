use super::stdout_line_wrap_meta;
use super::{ANSI_RESET, format_log_tag_inner, wrap_words_bounded};

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

pub fn print_stdout_line(who: &str, line: &str) {
    for para in line.split('\n') {
        let ts = super::timestamp_now_string();
        let ts = ts.as_str();
        let (max_payload, wrap) = stdout_line_wrap_meta(who, para);
        if !wrap {
            let (display, log) = super::stdout_tagged_display_and_log_line(who, para, Some(ts));
            super::print_stdout_rendered_line(&display, &log);
            continue;
        }
        for seg in wrap_words_bounded(max_payload, para) {
            let (display, log) =
                super::stdout_tagged_display_and_log_line(who, &seg, Some(ts));
            super::print_stdout_rendered_line(&display, &log);
        }
    }
}

pub fn print_stdout_text(who: &str, text: &str) {
    for line in super::logical_lines(text) {
        print_stdout_line(who, line);
    }
}

pub fn print_stdout_raw_line(line: &str) {
    print_stdout_raw_line_with_ts(line, None);
}

pub fn print_stdout_raw_line_with_ts(line: &str, ts: Option<&str>) {
    let (display, log) = super::stdout_log_pair::stdout_raw_display_and_log_line(line, ts);
    super::print_stdout_rendered_line(&display, &log);
}

#[cfg(test)]
mod tests {
    use super::{format_line_stdout, format_line_stdout_ansi};
    use crate::output::{LOG_TAG_INNER_WIDTH, format_log_tag_inner};

    #[test]
    fn stdout_line_omits_timestamp_prefix() {
        let inner = format_log_tag_inner("kpop");
        assert_eq!(
            format_line_stdout("kpop", "hello"),
            format!("[{inner}] hello")
        );
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
    fn stdout_display_and_log_splits_timestamp_for_disk() {
        let (display, log) =
            crate::output::stdout_log_pair::stdout_tagged_display_and_log_line("kpop", "payload", None);
        assert!(!display.starts_with("20"));
        assert!(log.starts_with("20"));
        assert!(log.contains("] payload"));
    }

    #[test]
    fn log_tag_inner_width_is_fifteen() {
        assert_eq!(LOG_TAG_INNER_WIDTH, 15);
        assert_eq!(format_log_tag_inner("x").chars().count(), 15);
    }
}
