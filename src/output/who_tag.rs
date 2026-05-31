//! Fixed-width who-tag formatting and log-line payload parsing.

/// Fixed width (Unicode scalars) for the who label in log lines (`…| …`).
pub const LOG_TAG_INNER_WIDTH: usize = 1;

/// General operational info (e.g. "Running kiss check").
pub const WHO_O: &str = "o";
/// Heartbeats.
pub const WHO_H: &str = "h";
/// Normal agent output.
pub const WHO_M: &str = "m";
/// Thinking / thought chunks.
pub const WHO_B: &str = "b";
/// Tool calls.
pub const WHO_T: &str = "t";
/// User input (prompt, command line).
pub const WHO_U: &str = "u";

#[must_use]
pub fn format_log_tag_inner(label: &str) -> String {
    label.chars().take(LOG_TAG_INNER_WIDTH).collect()
}

/// Fixed-width who label with trailing pipe delimiter (no space), e.g. `m|`.
#[must_use]
pub fn format_who_tag_delim(label: &str) -> String {
    format!("{}|", format_log_tag_inner(label))
}

/// Who-tag prefix before payload: `{delim} `.
#[must_use]
pub fn format_who_tag_prefix(label: &str) -> String {
    format!("{} ", format_who_tag_delim(label))
}

/// Map legacy outbound/inbound direction to the single-char who tag (no `>`/`<` stem).
#[must_use]
pub fn format_acp_directional_tag_prefix(direction: char, _stem: &str) -> String {
    match direction {
        '>' => WHO_U.to_string(),
        '<' => WHO_M.to_string(),
        _ => WHO_M.to_string(),
    }
}

pub(crate) fn payload_after_fixed_width_who_tag(line: &str) -> Option<&str> {
    let (tag_end, _) = line.char_indices().nth(LOG_TAG_INNER_WIDTH)?;
    line[tag_end..].strip_prefix("| ")
}

pub(crate) fn payload_after_fixed_width_bracket_tag(line: &str) -> Option<&str> {
    payload_after_fixed_width_who_tag(line)
}

const LOG_TIMESTAMP_LEN: usize = 19;

pub(crate) fn is_log_timestamp_token(token: &str) -> bool {
    let b = token.as_bytes();
    b.len() == LOG_TIMESTAMP_LEN
        && b[8] == b'.'
        && b[15] == b'.'
        && b[..8].iter().all(u8::is_ascii_digit)
        && b[9..15].iter().all(u8::is_ascii_digit)
        && b[16..].iter().all(u8::is_ascii_digit)
}

#[must_use]
pub fn is_command_prelude_line(line: &str) -> bool {
    const CMD: &str = "Command: ";
    if line.starts_with(CMD) {
        return true;
    }
    if let Some(payload) = payload_after_fixed_width_bracket_tag(line) {
        return payload.starts_with(CMD);
    }
    let Some((ts, rest)) = line.split_once(' ') else {
        return false;
    };
    if !is_log_timestamp_token(ts) {
        return false;
    }
    payload_after_fixed_width_bracket_tag(rest).is_some_and(|payload| payload.starts_with(CMD))
}

#[cfg(test)]
mod tests {
    use super::{
        format_acp_directional_tag_prefix, format_who_tag_prefix, payload_after_fixed_width_bracket_tag,
        WHO_M, WHO_U,
    };

    #[test]
    fn bracket_tag_alias_delegates_to_pipe_parser() {
        let prefix = format_who_tag_prefix(WHO_M);
        assert_eq!(
            payload_after_fixed_width_bracket_tag(&format!("{prefix}Command: x")),
            Some("Command: x")
        );
    }

    #[test]
    fn directional_prefix_maps_to_single_char_tags() {
        assert_eq!(format_acp_directional_tag_prefix('>', "kpop"), WHO_U);
        assert_eq!(format_acp_directional_tag_prefix('<', "kpop"), WHO_M);
    }
}
