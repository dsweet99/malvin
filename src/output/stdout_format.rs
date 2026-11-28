use super::{ANSI_RESET, ERROR_WHO, WARNING_WHO, WHO_B, format_who_tag_prefix, who_tag_ansi};
use crate::terminal_palette::{ANSI_BOLD, ansi_error, ansi_tool_name, ansi_warning, ansi_who_tag};

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
        super::WHO_A | super::WHO_P => {
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
