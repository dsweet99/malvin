#[must_use]
pub fn format_line_with_timestamp(ts: &str, who: &str, line: &str) -> String {
    super::stdout_log_pair::tagged_log_line(ts, who, line)
}

#[must_use]
pub fn format_line(who: &str, line: &str) -> String {
    super::stdout_log_pair::stdout_tagged_display_and_log_line(who, line, None).1
}
