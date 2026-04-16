use std::io::{IsTerminal, stderr, stdout};

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

#[must_use]
pub fn stdout_is_wrappable_terminal() -> bool {
    stdout().is_terminal()
}

fn line_wrap_meta(ts: &str, who: &str, line: &str, stream_is_tty: bool) -> (usize, bool) {
    let prefix_len = super::format_line_with_timestamp(ts, who, "").chars().count();
    let max_payload = terminal_columns().saturating_sub(prefix_len).max(1);
    let wrap = stream_is_tty && line.chars().count() > max_payload;
    (max_payload, wrap)
}

#[must_use]
pub fn stdout_line_wrap_meta(ts: &str, who: &str, line: &str) -> (usize, bool) {
    line_wrap_meta(ts, who, line, stdout_is_wrappable_terminal())
}

#[must_use]
pub fn stderr_line_wrap_meta(ts: &str, who: &str, line: &str) -> (usize, bool) {
    line_wrap_meta(ts, who, line, stderr().is_terminal())
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
    for ch in word.chars() {
        if cur.chars().count() >= max_payload_chars {
            out.push(std::mem::take(&mut cur));
        }
        cur.push(ch);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

#[must_use]
pub fn wrap_words_bounded(max_payload_chars: usize, text: &str) -> Vec<String> {
    if max_payload_chars == 0 {
        return vec![text.to_string()];
    }
    let mut lines = Vec::new();
    let mut cur_line = String::new();
    for word in text.split_whitespace() {
        for chunk in split_long_word(word, max_payload_chars) {
            let can_extend = cur_line.is_empty()
                || cur_line.chars().count() + 1 + chunk.chars().count() <= max_payload_chars;
            if !can_extend {
                lines.push(std::mem::take(&mut cur_line));
            }
            if !cur_line.is_empty() {
                cur_line.push(' ');
            }
            cur_line.push_str(&chunk);
        }
    }
    if !cur_line.is_empty() || lines.is_empty() {
        lines.push(cur_line);
    }
    lines
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
        unsafe {
            match prev {
                Some(v) => std::env::set_var("COLUMNS", v),
                None => std::env::remove_var("COLUMNS"),
            }
        }
    }

    #[test]
    fn kiss_stringify_terminal_wrap_symbols() {
        let _ = stringify!(super::terminal_columns);
        let _ = stringify!(super::stdout_is_wrappable_terminal);
        let _ = stringify!(super::stdout_line_wrap_meta);
        let _ = stringify!(super::stderr_line_wrap_meta);
        let _ = stringify!(super::wrap_words_bounded);
    }
}
