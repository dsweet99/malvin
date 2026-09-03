use super::{ANSI_RESET, ERROR_WHO, WARNING_WHO, WHO_B, format_who_tag_prefix};
use crate::terminal_palette::{ANSI_BOLD, ansi_error, ansi_tool_name, ansi_warning, ansi_who_tag};

pub(crate) use super::stdout_render::{flush_stdout_rendered_line, print_stdout_rendered_line};
pub(crate) use super::who_tag_ansi;

pub(crate) fn logical_lines(text: &str) -> impl Iterator<Item = &str> {
    text.split_inclusive('\n')
        .map(|part| part.strip_suffix('\n').unwrap_or(part))
}

#[must_use]
pub fn format_line_stdout(who: &str, line: &str) -> String {
    format!("{}{line}", format_who_tag_prefix(who))
}

#[must_use]
pub fn format_line_stdout_ansi(who: &str, line: &str) -> String {
    let prefix = format_who_tag_prefix(who);
    match who {
        WARNING_WHO => format!("{}{prefix}{line}{ANSI_RESET}", ansi_warning()),
        ERROR_WHO => format!("{}{prefix}{line}{ANSI_RESET}", ansi_error()),
        WHO_B => format!("{}{prefix}{line}{ANSI_RESET}", super::ANSI_DIM),
        super::WHO_A => {
            let tag_color = who_tag_ansi(who);
            format!(
                "{tag_color}{prefix}{ANSI_RESET}{ANSI_BOLD}{}{line}{ANSI_RESET}",
                ansi_tool_name()
            )
        }
        _ => {
            let tag_color = who_tag_ansi(who);
            format!("{tag_color}{prefix}{ANSI_RESET}{line}")
        }
    }
}

#[must_use]
pub fn format_heartbeat_stdout_ansi(who: &str, line: &str) -> String {
    format!(
        "{}{}{line}{ANSI_RESET}",
        ansi_who_tag(),
        super::format_who_tag_delim(who)
    )
}

pub fn print_stdout_line(who: &str, line: &str) {
    super::stdout_display_md::print_stdout_line_with_markdown(who, line, false);
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

#[allow(dead_code)]
pub(crate) fn flush_stdout_raw_line_with_ts(line: &str, ts: Option<&str>) {
    let (display, log) = super::stdout_log_pair::stdout_raw_display_and_log_line(line, ts);
    flush_stdout_rendered_line(&display, &log);
}

#[cfg(test)]
mod tests {
    use super::{format_line_stdout, format_line_stdout_ansi};
    use crate::output::{
        LOG_TAG_INNER_WIDTH, WHO_H, WHO_M, format_log_tag_inner, format_who_tag_prefix,
    };

    #[test]
    fn stdout_line_omits_timestamp_prefix() {
        let prefix = format_who_tag_prefix(WHO_M);
        assert_eq!(format_line_stdout(WHO_M, "hello"), format!("{prefix}hello"));
        assert!(!format_line_stdout(WHO_M, "hello").starts_with("20"));
    }

    #[test]
    fn stdout_ansi_line_omits_timestamp_prefix() {
        let plain = format_line_stdout(WHO_M, "hello");
        let ansi = format_line_stdout_ansi(WHO_M, "hello");
        assert!(ansi.contains('\x1b'));
        assert!(ansi.ends_with("hello"));
        assert!(!plain.contains('\x1b'));
    }

    #[test]
    fn stdout_display_and_log_splits_timestamp_for_disk() {
        let (display, log) = crate::output::stdout_log_pair::stdout_tagged_display_and_log_line(
            WHO_M, "payload", None,
        );
        assert!(!display.starts_with("20"));
        assert!(log.starts_with("20"));
        assert!(log.contains("|payload"));
    }

    #[test]
    fn log_tag_inner_width_is_one() {
        assert_eq!(LOG_TAG_INNER_WIDTH, 1);
        assert_eq!(format_log_tag_inner("x").chars().count(), 1);
    }

    #[test]
    fn heartbeat_display_omits_timestamp_on_stdout() {
        use std::time::{Duration, Instant};

        use crate::output::is_log_timestamp_token;
        use crate::output::stdout_heartbeat::{
            heartbeat_rendered_if_due, reset_stdout_heartbeat_for_test,
            test_set_last_heartbeat_elapsed,
        };

        reset_stdout_heartbeat_for_test();
        test_set_last_heartbeat_elapsed(Duration::from_secs(61));
        let (display, log) =
            heartbeat_rendered_if_due(Instant::now(), false).expect("heartbeat due");
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
        use crate::output::{init_stdout_style_for_test, stdout_use_color};

        init_stdout_style_for_test(false);
        let (display, _) = stdout_heartbeat_display_and_log_line(
            WHO_H,
            "HB: 20260524.000000",
            Some("20260524.000000.000"),
        );
        let expected = if stdout_use_color() {
            super::format_heartbeat_stdout_ansi(WHO_H, "HB: 20260524.000000")
        } else {
            format_line_stdout(WHO_H, "HB: 20260524.000000")
        };
        assert_eq!(display, expected);
        if stdout_use_color() {
            assert!(display.contains('\x1b'));
        }
    }

    #[test]
    fn heartbeat_ansi_display_uses_uniform_who_tag_color() {
        use crate::terminal_palette::ansi_who_tag;

        let payload = "20260524.000000 Still alive.";
        let (display, log) =
            crate::output::stdout_log_pair::heartbeat_display_and_log_line_for_color(
                WHO_H,
                payload,
                Some("20260524.000000.000"),
                true,
            );
        let expected = super::format_heartbeat_stdout_ansi(WHO_H, payload);
        assert_eq!(display, expected);
        assert!(display.contains('\x1b'));
        assert!(!display.starts_with("20"));
        assert!(log.starts_with("20260524.000000.000"));
        assert!(
            display.starts_with(ansi_who_tag()),
            "heartbeat line must open with who_tag color; got {display:?}"
        );
        let reset_before_payload = format!("|{}{payload}", super::ANSI_RESET);
        assert!(
            !display.contains(&reset_before_payload),
            "must not reset color before payload"
        );
        assert!(display.ends_with(super::ANSI_RESET));
    }

    #[test]
    fn agent_start_ansi_payload_uses_tool_name_color() {
        use crate::output::WHO_A;
        use crate::terminal_palette::{ANSI_BOLD, ANSI_RESET, ansi_tool_name, ansi_who_tag};

        let line = format_line_stdout_ansi(WHO_A, "cursor:auto");
        let who = ansi_who_tag();
        let tool = format!("{ANSI_BOLD}{}", ansi_tool_name());
        assert!(
            line.starts_with(who),
            "a| who-tag must use who_tag color; got {line:?}"
        );
        let tool_pos = line.find(&tool).expect("bold tool_name color on a| line");
        let payload_pos = line.find("cursor:auto").expect("payload");
        assert!(
            tool_pos < payload_pos,
            "provider:model must match tool-name brightness: {line:?}"
        );
        assert!(
            line[tool_pos..].contains(&format!("cursor:auto{ANSI_RESET}")),
            "payload must sit inside tool_name span: {line:?}"
        );
        assert_ne!(
            who,
            ansi_tool_name(),
            "who_tag and tool_name slots must differ"
        );
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
