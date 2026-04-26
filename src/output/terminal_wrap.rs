use std::io::{IsTerminal, stderr, stdout};

use crate::ansi_strip::strip_ansi_escapes;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

fn columns_from_env() -> Option<usize> {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|&c| (20..=500).contains(&c))
}

fn columns_from_tty() -> Option<usize> {
    let (w, _) = terminal_size::terminal_size()?;
    let w = usize::from(w.0);
    if w < 20 {
        return None;
    }
    Some(w.min(500))
}

#[must_use]
pub fn terminal_columns() -> usize {
    columns_from_env().or_else(columns_from_tty).unwrap_or(80)
}

/// True when stdout is a TTY, or `COLUMNS` parses to a column count in `20..=500`, so long log
/// lines may be word-wrapped to the available width.
#[must_use]
pub fn stdout_allows_log_word_wrap() -> bool {
    stdout().is_terminal() || columns_from_env().is_some()
}

/// True when stderr is a TTY, or `COLUMNS` is valid the same way as for [`stdout_allows_log_word_wrap`].
#[must_use]
pub fn stderr_allows_log_word_wrap() -> bool {
    stderr().is_terminal() || columns_from_env().is_some()
}

fn malvin_tagged_stdout_prefix_len(ts: &str, who: &str) -> usize {
    let s = if super::stdout_use_color() {
        super::format_line_with_timestamp_ansi(ts, who, "")
    } else {
        super::format_line_with_timestamp(ts, who, "")
    };
    strip_ansi_escapes(&s).chars().count()
}

#[must_use]
pub fn line_wrap_for_prefix_len(
    prefix_len: usize,
    line: &str,
    allow_word_wrap: bool,
) -> (usize, bool) {
    let max_payload = terminal_columns().saturating_sub(prefix_len).max(1);
    let wrap = allow_word_wrap && line.width() > max_payload;
    (max_payload, wrap)
}

fn line_wrap_meta_tagged_malvin(
    ts: &str,
    who: &str,
    line: &str,
    allow_word_wrap: bool,
) -> (usize, bool) {
    let prefix_len = malvin_tagged_stdout_prefix_len(ts, who);
    line_wrap_for_prefix_len(prefix_len, line, allow_word_wrap)
}

fn line_wrap_meta_tagged_plain(
    ts: &str,
    who: &str,
    line: &str,
    allow_word_wrap: bool,
) -> (usize, bool) {
    let prefix_len = super::format_line_with_timestamp(ts, who, "")
        .chars()
        .count();
    line_wrap_for_prefix_len(prefix_len, line, allow_word_wrap)
}

#[must_use]
pub fn stdout_line_wrap_meta(ts: &str, who: &str, line: &str) -> (usize, bool) {
    line_wrap_meta_tagged_malvin(ts, who, line, stdout_allows_log_word_wrap())
}

#[must_use]
pub fn stderr_line_wrap_meta(ts: &str, who: &str, line: &str) -> (usize, bool) {
    line_wrap_meta_tagged_plain(ts, who, line, stderr_allows_log_word_wrap())
}

fn char_display_cell(ch: char) -> usize {
    let w = UnicodeWidthChar::width(ch).unwrap_or(0);
    if w == 0 && !ch.is_whitespace() { 1 } else { w }
}

#[must_use]
fn wrap_words_single_line_no_newlines(max_display_width: usize, text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut width_prefix: Vec<usize> = vec![0; chars.len() + 1];
    for i in 0..chars.len() {
        width_prefix[i + 1] = width_prefix[i].saturating_add(char_display_cell(chars[i]));
    }
    let mut lines = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        if width_prefix[chars.len()] - width_prefix[start] <= max_display_width {
            lines.push(chars[start..].iter().collect());
            break;
        }
        let mut end = start;
        while end < chars.len()
            && width_prefix[end + 1].saturating_sub(width_prefix[start]) <= max_display_width
        {
            end += 1;
        }
        if end == start {
            end = start + 1;
        }
        let split = chars[start..end]
            .iter()
            .rposition(|ch| ch.is_whitespace())
            .map_or(end, |idx| {
                let mut split = start + idx + 1;
                while split < end && chars[split].is_whitespace() {
                    split += 1;
                }
                split
            });
        let split = if split > start { split } else { end };
        lines.push(chars[start..split].iter().collect());
        start = split;
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}

#[must_use]
pub fn wrap_words_bounded(max_payload_chars: usize, text: &str) -> Vec<String> {
    if max_payload_chars == 0 {
        return vec![text.to_string()];
    }
    let mut out = Vec::new();
    for para in text.split('\n') {
        if para.is_empty() {
            out.push(String::new());
            continue;
        }
        out.extend(wrap_words_single_line_no_newlines(max_payload_chars, para));
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::{terminal_columns, wrap_words_bounded};

    static COLUMNS_TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn wrap_fits_short_line_single_segment() {
        let v = wrap_words_bounded(72, "hello world");
        assert_eq!(v, vec!["hello world"]);
    }

    #[test]
    fn wrap_does_not_merge_words_across_line_breaks() {
        let v = wrap_words_bounded(40, "aa bb\ncc dd");
        assert_eq!(v, vec!["aa bb", "cc dd"]);
    }

    #[test]
    fn wrap_splits_at_word_boundary() {
        let v = wrap_words_bounded(10, "one two three four five");
        assert!(
            v.iter()
                .all(|s| unicode_width::UnicodeWidthStr::width(s.as_str()) <= 10)
        );
        assert_eq!(v.concat(), "one two three four five");
    }

    #[test]
    fn wrap_preserves_repeated_spaces() {
        let v = wrap_words_bounded(6, "aa  bb  cc");
        assert_eq!(v, vec!["aa  ", "bb  cc"]);
        assert_eq!(v.concat(), "aa  bb  cc");
    }

    #[test]
    fn wrap_preserves_leading_indent() {
        let v = wrap_words_bounded(8, "    code block");
        assert_eq!(v, vec!["    ", "code ", "block"]);
        assert_eq!(v.concat(), "    code block");
    }

    #[test]
    fn long_word_splits_by_chars() {
        let w = "a".repeat(25);
        let v = wrap_words_bounded(10, &w);
        assert_eq!(v.len(), 3);
        assert!(
            v.iter()
                .all(|s| unicode_width::UnicodeWidthStr::width(s.as_str()) <= 10)
        );
        assert_eq!(v.concat(), w);
    }

    #[test]
    fn wrap_splits_wide_emoji_by_display_width() {
        let cell = "😀";
        assert_eq!(unicode_width::UnicodeWidthStr::width(cell), 2);
        let line = [cell; 6].join("");
        let v = wrap_words_bounded(10, &line);
        assert!(
            v.len() >= 2,
            "six width-2 cells exceed width 10, expected multiple segments: {v:?}"
        );
        assert!(
            v.iter()
                .all(|s| unicode_width::UnicodeWidthStr::width(s.as_str()) <= 10),
            "{v:?}"
        );
        assert_eq!(v.concat(), line);
    }

    #[allow(unsafe_code)]
    #[test]
    fn terminal_columns_env_override() {
        let _guard = COLUMNS_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let prev = std::env::var("COLUMNS").ok();
        unsafe {
            std::env::set_var("COLUMNS", "100");
        }
        assert_eq!(terminal_columns(), 100);
        assert!(super::stdout_allows_log_word_wrap());
        assert!(super::stderr_allows_log_word_wrap());
        unsafe {
            match prev {
                Some(v) => std::env::set_var("COLUMNS", v),
                None => std::env::remove_var("COLUMNS"),
            }
        }
    }

    #[test]
    #[allow(unsafe_code)]
    fn long_line_wraps_when_valid_columns_env_set() {
        let _guard = COLUMNS_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let prev = std::env::var("COLUMNS").ok();
        let long = "x".repeat(400);
        unsafe {
            std::env::set_var("COLUMNS", "40");
        }
        let (_max, wrap) = super::stdout_line_wrap_meta("20260101.120000.000", "malvin", &long);
        assert!(
            super::stdout_allows_log_word_wrap() && wrap,
            "COLUMNS set should allow wrapping a long line even when stdout is not a TTY"
        );
        unsafe {
            match prev {
                Some(v) => std::env::set_var("COLUMNS", v),
                None => std::env::remove_var("COLUMNS"),
            }
        }
    }

    #[test]
    fn kiss_stringify_terminal_wrap_symbols() {
        let _ = (
            stringify!(super::line_wrap_for_prefix_len),
            stringify!(super::terminal_columns),
            stringify!(super::stdout_allows_log_word_wrap),
            stringify!(super::stderr_allows_log_word_wrap),
            stringify!(super::stdout_line_wrap_meta),
            stringify!(super::stderr_line_wrap_meta),
            stringify!(super::wrap_words_bounded),
            stringify!(super::wrap_words_single_line_no_newlines),
            stringify!(super::char_display_cell),
            stringify!(super::columns_from_env),
            stringify!(super::columns_from_tty),
            stringify!(super::malvin_tagged_stdout_prefix_len),
            stringify!(super::line_wrap_meta_tagged_malvin),
            stringify!(super::line_wrap_meta_tagged_plain),
        );
    }
}
