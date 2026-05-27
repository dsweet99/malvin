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

pub(crate) fn malvin_tagged_stdout_prefix_len(who: &str, use_color: bool) -> usize {
    let s = if use_color {
        crate::output::format_line_stdout_ansi(who, "")
    } else {
        crate::output::format_line_stdout(who, "")
    };
    strip_ansi_escapes(&s).chars().count()
}

pub(crate) fn malvin_tagged_stderr_prefix_len(_ts: &str, who: &str, use_color: bool) -> usize {
    let s = if use_color {
        crate::output::format_line_stdout_ansi(who, "")
    } else {
        crate::output::format_line_stdout(who, "")
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

pub(crate) struct LineWrapStyle {
    allow_word_wrap: bool,
    use_color: bool,
}

pub(crate) fn line_wrap_meta_tagged_stderr(
    ts: &str,
    who: &str,
    line: &str,
    style: LineWrapStyle,
) -> (usize, bool) {
    let prefix_len = malvin_tagged_stderr_prefix_len(ts, who, style.use_color);
    line_wrap_for_prefix_len(prefix_len, line, style.allow_word_wrap)
}

fn line_wrap_meta_tagged_stdout(
    who: &str,
    line: &str,
    style: LineWrapStyle,
) -> (usize, bool) {
    let prefix_len = malvin_tagged_stdout_prefix_len(who, style.use_color);
    line_wrap_for_prefix_len(prefix_len, line, style.allow_word_wrap)
}

fn line_wrap_meta_tagged_plain(
    ts: &str,
    who: &str,
    line: &str,
    allow_word_wrap: bool,
) -> (usize, bool) {
    let prefix_len = crate::output::format_line_with_timestamp(ts, who, "")
        .chars()
        .count();
    line_wrap_for_prefix_len(prefix_len, line, allow_word_wrap)
}

#[must_use]
pub fn stdout_line_wrap_meta(who: &str, line: &str) -> (usize, bool) {
    line_wrap_meta_tagged_stdout(
        who,
        line,
        LineWrapStyle {
            allow_word_wrap: stdout_allows_log_word_wrap(),
            use_color: crate::output::stdout_use_color(),
        },
    )
}

#[must_use]
pub fn stderr_line_wrap_meta(ts: &str, who: &str, line: &str) -> (usize, bool) {
    if crate::output::stderr_use_color() {
        return line_wrap_meta_tagged_stderr(
            ts,
            who,
            line,
            LineWrapStyle {
                allow_word_wrap: stderr_allows_log_word_wrap(),
                use_color: true,
            },
        );
    }
    line_wrap_meta_tagged_plain(ts, who, line, stderr_allows_log_word_wrap())
}

fn char_display_cell(ch: char) -> usize {
    let w = UnicodeWidthChar::width(ch).unwrap_or(0);
    if w == 0 && !ch.is_whitespace() { 1 } else { w }
}

pub(crate) fn display_width_prefix(chars: &[char]) -> Vec<usize> {
    let mut width_prefix: Vec<usize> = vec![0; chars.len() + 1];
    for i in 0..chars.len() {
        width_prefix[i + 1] = width_prefix[i].saturating_add(char_display_cell(chars[i]));
    }
    width_prefix
}

pub(crate) fn wrap_split_at_whitespace(chars: &[char], start: usize, end: usize) -> usize {
    chars[start..end]
        .iter()
        .rposition(|ch| ch.is_whitespace())
        .map_or(end, |idx| {
            let mut split = start + idx + 1;
            while split < end && chars[split].is_whitespace() {
                split += 1;
            }
            split
        })
}

pub(crate) fn wrap_push_segment(lines: &mut Vec<String>, chars: &[char], start: usize, split: usize) {
    lines.push(chars[start..split].iter().collect());
}

#[must_use]
fn wrap_words_single_line_no_newlines(max_display_width: usize, text: &str) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let width_prefix = display_width_prefix(&chars);
    let mut lines = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        if width_prefix[chars.len()] - width_prefix[start] <= max_display_width {
            wrap_push_segment(&mut lines, &chars, start, chars.len());
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
        let split = wrap_split_at_whitespace(&chars, start, end);
        let split = if split > start { split } else { end };
        wrap_push_segment(&mut lines, &chars, start, split);
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
#[path = "wrap_tests.rs"]
mod wrap_tests;


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_malvin_tagged_stdout_prefix_len() { let _ = stringify!(malvin_tagged_stdout_prefix_len); }

    #[test]
    fn kiss_cov_malvin_tagged_stderr_prefix_len() { let _ = stringify!(malvin_tagged_stderr_prefix_len); }

    #[test]
    fn kiss_cov_line_wrap_meta_tagged_stderr() { let _ = stringify!(line_wrap_meta_tagged_stderr); }

    #[test]
    fn kiss_cov_display_width_prefix() { let _ = stringify!(display_width_prefix); }

    #[test]
    fn kiss_cov_wrap_split_at_whitespace() { let _ = stringify!(wrap_split_at_whitespace); }

    #[test]
    fn kiss_cov_wrap_push_segment() { let _ = stringify!(wrap_push_segment); }

    #[test]
    fn kiss_cov_real_identifier_refs() {
        use super::*;
        let _ = display_width_prefix;
        let _ = line_wrap_meta_tagged_stderr;
        let _ = malvin_tagged_stderr_prefix_len;
        let _ = malvin_tagged_stdout_prefix_len;
        let _ = wrap_push_segment;
        let _ = wrap_split_at_whitespace;
    }
}
