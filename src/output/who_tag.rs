pub const LOG_TAG_INNER_WIDTH: usize = 1;

pub const WHO_O: &str = "o";
pub const WHO_H: &str = "h";
pub const WHO_M: &str = "m";
pub const WHO_B: &str = "b";
pub const WHO_T: &str = "t";
pub const WHO_U: &str = "u";

#[must_use]
pub fn format_log_tag_inner(label: &str) -> String {
    label.chars().take(LOG_TAG_INNER_WIDTH).collect()
}

#[must_use]
pub fn format_who_tag_delim(label: &str) -> String {
    format!("{}|", format_log_tag_inner(label))
}

#[must_use]
pub(crate) fn who_tag_display_space_after_pipe(label: &str) -> bool {
    matches!(format_log_tag_inner(label).as_str(), WHO_B | WHO_T)
}

#[must_use]
pub fn format_who_tag_prefix(label: &str) -> String {
    let delim = format_who_tag_delim(label);
    if who_tag_display_space_after_pipe(label) {
        format!("{delim} ")
    } else {
        delim
    }
}

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
    let rest = line[tag_end..].strip_prefix('|')?;
    Some(rest.strip_prefix(' ').unwrap_or(rest))
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
        WHO_M, WHO_U, format_acp_directional_tag_prefix, format_who_tag_prefix,
        payload_after_fixed_width_bracket_tag,
    };

    #[test]
    fn who_tag_display_space_after_pipe_classifies_tags() {
        use super::{WHO_B, WHO_H, WHO_M, WHO_O, WHO_T, WHO_U, who_tag_display_space_after_pipe};

        assert!(who_tag_display_space_after_pipe(WHO_B));
        assert!(who_tag_display_space_after_pipe(WHO_T));
        assert!(!who_tag_display_space_after_pipe(WHO_M));
        assert!(!who_tag_display_space_after_pipe(WHO_O));
        assert!(!who_tag_display_space_after_pipe(WHO_H));
        assert!(!who_tag_display_space_after_pipe(WHO_U));
    }

    #[test]
    fn display_prefix_adds_space_after_pipe_for_thought_and_tool_only() {
        use super::{WHO_B, WHO_M, WHO_O, WHO_T, format_who_tag_delim};

        assert_eq!(format_who_tag_prefix(WHO_B), "b| ");
        assert_eq!(format_who_tag_prefix(WHO_T), "t| ");
        assert_eq!(format_who_tag_prefix(WHO_M), format_who_tag_delim(WHO_M));
        assert_eq!(format_who_tag_prefix(WHO_O), format_who_tag_delim(WHO_O));
    }

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
        assert_eq!(format_acp_directional_tag_prefix('>', "router"), WHO_U);
        assert_eq!(format_acp_directional_tag_prefix('<', "router"), WHO_M);
    }
}
