use super::stdout_line_wrap_meta;
use super::{ANSI_RESET, format_log_tag_inner, wrap_words_bounded};
use crate::terminal_palette::ANSI_TOOL_NAVY;

pub(crate) use super::who_tag_ansi;
pub(crate) use super::stdout_render::{flush_stdout_rendered_line, print_stdout_rendered_line};

pub(crate) fn logical_lines(text: &str) -> impl Iterator<Item = &str> {
    text.split_inclusive('\n')
        .map(|part| part.strip_suffix('\n').unwrap_or(part))
}

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

/// Heartbeat TTY lines keep the who-tag color through the payload (no reset after `]`).
#[must_use]
pub fn format_heartbeat_stdout_ansi(who: &str, line: &str) -> String {
    let inner = format_log_tag_inner(who);
    format!("{ANSI_TOOL_NAVY}[{inner}] {line}{ANSI_RESET}")
}

pub fn print_stdout_line(who: &str, line: &str) {
    for para in line.split('\n') {
        let ts = super::timestamp_now_string();
        let ts = ts.as_str();
        let (max_payload, wrap) = stdout_line_wrap_meta(who, para);
        if !wrap {
            let (display, log) = super::stdout_tagged_display_and_log_line(who, para, Some(ts));
            print_stdout_rendered_line(&display, &log);
            continue;
        }
        for seg in wrap_words_bounded(max_payload, para) {
            let (display, log) = super::stdout_tagged_display_and_log_line(who, &seg, Some(ts));
            print_stdout_rendered_line(&display, &log);
        }
    }
}

pub fn print_stdout_text(who: &str, text: &str) {
    for line in logical_lines(text) {
        print_stdout_line(who, line);
    }
}

pub fn print_stdout_raw_line(line: &str) {
    print_stdout_raw_line_with_ts(line, None);
}

pub fn print_stdout_raw_line_with_ts(line: &str, ts: Option<&str>) {
    let (display, log) = super::stdout_log_pair::stdout_raw_display_and_log_line(line, ts);
    print_stdout_rendered_line(&display, &log);
}

pub(crate) fn flush_stdout_raw_line_with_ts(line: &str, ts: Option<&str>) {
    let (display, log) = super::stdout_log_pair::stdout_raw_display_and_log_line(line, ts);
    flush_stdout_rendered_line(&display, &log);
}

#[cfg(test)]
mod tests {
    use super::{format_line_stdout, format_line_stdout_ansi};
    use crate::output::{LOG_TAG_INNER_WIDTH, format_log_tag_inner, MALVIN_WHO};

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
        let (display, log) = crate::output::stdout_log_pair::stdout_tagged_display_and_log_line(
            "kpop", "payload", None,
        );
        assert!(!display.starts_with("20"));
        assert!(log.starts_with("20"));
        assert!(log.contains("] payload"));
    }

    #[test]
    fn log_tag_inner_width_is_fifteen() {
        assert_eq!(LOG_TAG_INNER_WIDTH, 15);
        assert_eq!(format_log_tag_inner("x").chars().count(), 15);
    }

    #[test]
    fn heartbeat_display_omits_timestamp_on_stdout() {
        use std::time::{Duration, Instant};

        use crate::output::stdout_heartbeat::{
            heartbeat_rendered_if_due, reset_stdout_heartbeat_for_test, test_set_last_heartbeat_elapsed,
        };
        use crate::output::is_log_timestamp_token;

        reset_stdout_heartbeat_for_test();
        test_set_last_heartbeat_elapsed(Duration::from_secs(61));
        let (display, log) = heartbeat_rendered_if_due(Instant::now(), false).expect("heartbeat due");
        assert!(
            !display.starts_with("20"),
            "stdout display must omit wall-clock prefix; got {display:?}"
        );
        let ts = log.split_whitespace().next().expect("log timestamp");
        assert!(is_log_timestamp_token(ts));
    }

    #[test]
    fn heartbeat_display_matches_logger_format_for_color_mode() {
        use crate::output::stdout_heartbeat_display_and_log_line;
        use crate::output::{init_stdout_style, stdout_use_color};

        init_stdout_style(true);
        let (display, _) =
            stdout_heartbeat_display_and_log_line(MALVIN_WHO, "HB: 20260524.000000", Some("20260524.000000.000"));
        let expected = if stdout_use_color() {
            super::format_heartbeat_stdout_ansi(MALVIN_WHO, "HB: 20260524.000000")
        } else {
            format_line_stdout(MALVIN_WHO, "HB: 20260524.000000")
        };
        assert_eq!(display, expected);
        if stdout_use_color() {
            assert!(display.contains('\x1b'));
        }
    }

    #[test]
    fn heartbeat_ansi_display_uses_uniform_who_tag_color() {
        use crate::terminal_palette::ANSI_TOOL_NAVY;

        let payload = "20260524.000000 Still alive.";
        let (display, log) = crate::output::stdout_log_pair::heartbeat_display_and_log_line_for_color(
            MALVIN_WHO,
            payload,
            Some("20260524.000000.000"),
            true,
        );
        let expected = super::format_heartbeat_stdout_ansi(MALVIN_WHO, payload);
        assert_eq!(display, expected);
        assert!(display.contains('\x1b'));
        assert!(!display.starts_with("20"));
        assert!(log.starts_with("20260524.000000.000"));
        assert!(
            display.starts_with(ANSI_TOOL_NAVY),
            "heartbeat line must open with navy who-tag; got {display:?}"
        );
        let reset_before_payload = format!("]{}{} ", super::ANSI_RESET, payload);
        assert!(
            !display.contains(&reset_before_payload),
            "must not reset color before payload"
        );
        assert!(display.ends_with(super::ANSI_RESET));
    }

    #[test]
    fn heartbeat_rendered_if_due_covers_arm_and_due_paths() {
        use std::time::{Duration, Instant};

        use crate::output::stdout_heartbeat::{
            heartbeat_due, heartbeat_rendered_if_due, reset_stdout_heartbeat_for_test,
            test_set_last_heartbeat_elapsed,
        };

        let now = Instant::now();
        assert!(!heartbeat_due(now, now));
        reset_stdout_heartbeat_for_test();
        assert!(heartbeat_rendered_if_due(Instant::now(), false).is_none());
        assert!(heartbeat_rendered_if_due(Instant::now(), true).is_none());
        assert!(heartbeat_rendered_if_due(Instant::now(), false).is_none());
        test_set_last_heartbeat_elapsed(Duration::from_secs(61));
        assert!(heartbeat_rendered_if_due(Instant::now(), false).is_some());
    }
}
