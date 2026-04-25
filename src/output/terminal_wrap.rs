use std::io::{IsTerminal, stderr, stdout};

use crate::ansi_strip::strip_ansi_escapes;

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
    columns_from_env()
        .or_else(columns_from_tty)
        .unwrap_or(80)
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

pub fn line_wrap_for_prefix_len(
    prefix_len: usize,
    line: &str,
    allow_word_wrap: bool,
) -> (usize, bool) {
    let max_payload = terminal_columns().saturating_sub(prefix_len).max(1);
    let wrap = allow_word_wrap && line.chars().count() > max_payload;
    (max_payload, wrap)
}

fn line_wrap_meta_tagged_malvin(ts: &str, who: &str, line: &str, allow_word_wrap: bool) -> (usize, bool) {
    let prefix_len = malvin_tagged_stdout_prefix_len(ts, who);
    line_wrap_for_prefix_len(prefix_len, line, allow_word_wrap)
}

fn line_wrap_meta_tagged_plain(ts: &str, who: &str, line: &str, allow_word_wrap: bool) -> (usize, bool) {
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

fn split_long_word(word: &str, max_payload_chars: usize) -> Vec<String> {
    if max_payload_chars == 0 {
        return vec![word.to_string()];
    }
    if word.chars().count() <= max_payload_chars {
        return vec![word.to_string()];
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut cur_len: usize = 0;
    for ch in word.chars() {
        if cur_len >= max_payload_chars {
            out.push(std::mem::take(&mut cur));
            cur_len = 0;
        }
        cur.push(ch);
        cur_len += 1;
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[must_use]
fn wrap_words_single_line_no_newlines(max_payload_chars: usize, text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur_line = String::new();
    let mut cur_len: usize = 0;
    for word in text.split_whitespace() {
        for chunk in split_long_word(word, max_payload_chars) {
            let chunk_n = chunk.chars().count();
            let can_extend = cur_line.is_empty() || cur_len + 1 + chunk_n <= max_payload_chars;
            if !can_extend {
                lines.push(std::mem::take(&mut cur_line));
                cur_len = 0;
            }
            if !cur_line.is_empty() {
                cur_line.push(' ');
                cur_len += 1;
            }
            cur_line.push_str(&chunk);
            cur_len += chunk_n;
        }
    }
    if !cur_line.is_empty() || lines.is_empty() {
        lines.push(cur_line);
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

    use super::{split_long_word, terminal_columns, wrap_words_bounded};

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
        assert!(v.iter().all(|s| s.chars().count() <= 10));
        assert_eq!(v.join(" "), "one two three four five");
    }

    #[test]
    fn long_word_splits_by_chars() {
        let w = "a".repeat(25);
        let v = wrap_words_bounded(10, &w);
        assert_eq!(v.len(), 3);
        assert!(v.iter().all(|s| s.chars().count() <= 10));
        assert_eq!(v.concat(), w);
    }

    #[test]
    fn split_long_word_respects_boundary() {
        let v = split_long_word("abcdefghij", 4);
        assert_eq!(v, vec!["abcd", "efgh", "ij"]);
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
            stringify!(super::columns_from_env),
            stringify!(super::columns_from_tty),
            stringify!(super::malvin_tagged_stdout_prefix_len),
            stringify!(super::line_wrap_meta_tagged_malvin),
            stringify!(super::line_wrap_meta_tagged_plain),
        );
    }
}
