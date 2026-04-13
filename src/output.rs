//! Shared line-oriented formatting for stdout, stderr, and run logs.

use chrono::Local;

pub const MALVIN_WHO: &str = "malvin";
pub const LEARNING_PLACEHOLDER: &str = "[learning...]";

#[must_use]
pub fn format_line_with_timestamp(ts: &str, who: &str, line: &str) -> String {
    format!("{ts}:[{who}]: {line}")
}

#[must_use]
pub fn format_line(who: &str, line: &str) -> String {
    let now = Local::now();
    let ts = format!(
        "{}.{:03}",
        now.format("%Y%m%d.%H%M%S"),
        now.timestamp_subsec_millis()
    );
    format_line_with_timestamp(&ts, who, line)
}

pub fn print_stdout_line(who: &str, line: &str) {
    println!("{}", format_line(who, line));
}

pub fn print_stderr_line(who: &str, line: &str) {
    eprintln!("{}", format_line(who, line));
}

pub fn print_stdout_text(who: &str, text: &str) {
    for line in logical_lines(text) {
        print_stdout_line(who, line);
    }
}

#[must_use]
pub fn is_command_prelude_line(line: &str) -> bool {
    line.starts_with("Command: ")
        || line
            .split_once("]: ")
            .is_some_and(|(_, payload)| payload.starts_with("Command: "))
}

fn logical_lines(text: &str) -> impl Iterator<Item = &str> {
    text.split_inclusive('\n')
        .map(|part| part.strip_suffix('\n').unwrap_or(part))
}

#[cfg(test)]
mod tests {
    use super::{
        LEARNING_PLACEHOLDER, MALVIN_WHO, format_line_with_timestamp, is_command_prelude_line,
    };

    #[test]
    fn formats_expected_mini_header() {
        assert_eq!(
            format_line_with_timestamp("20260413.121314.015", "kpop", "hello"),
            "20260413.121314.015:[kpop]: hello"
        );
    }

    #[test]
    fn detects_prefixed_and_unprefixed_command_prelude() {
        assert!(is_command_prelude_line("Command: malvin code @plan.md"));
        assert!(is_command_prelude_line(
            "20260413.121314.015:[malvin]: Command: malvin code @plan.md"
        ));
        assert!(!is_command_prelude_line(
            "20260413.121314.015:[kpop]: not a command line"
        ));
    }

    #[test]
    fn exported_constants_match_public_contract() {
        assert_eq!(MALVIN_WHO, "malvin");
        assert_eq!(LEARNING_PLACEHOLDER, "[learning...]");
    }
}
