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
        assert_eq!(super::columns_from_env(), Some(100));
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
        let (_max, wrap) = super::stdout_line_wrap_meta("malvin", &long);
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
    fn line_wrap_meta_exercises_remaining_helpers() {
        let (max, _) = super::line_wrap_for_prefix_len(8, "hello world", true);
        assert!(max > 0);
        let v = super::wrap_words_single_line_no_newlines(20, "hello world");
        assert_eq!(v, vec!["hello world"]);
        assert_eq!(super::char_display_cell('a'), 1);
        let (tag_max, _) =
            super::line_wrap_meta_tagged_stdout("malvin", "line", true, false);
        assert!(tag_max > 0);
        let (pmax, _) = super::line_wrap_meta_tagged_plain("t", "who", "line", true);
        assert!(pmax > 0);
        let (smax, _) = super::stderr_line_wrap_meta("t", "w", "line");
        assert!(smax > 0);
        let _ = super::columns_from_tty();
    }
}
