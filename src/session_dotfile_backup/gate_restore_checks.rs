pub(super) fn substantive_check_lines(bytes: &[u8]) -> Vec<String> {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Vec::new();
    };
    crate::repo_gates::parse_malvin_checks_text(text)
}

#[cfg(test)]
mod tests {
    use super::substantive_check_lines;

    #[test]
    fn substantive_check_lines_skips_comments_and_blank_lines() {
        assert_eq!(
            substantive_check_lines(b"# header\n\nmake lint\n"),
            vec!["make lint".to_string()]
        );
    }

    #[test]
    fn substantive_check_lines_returns_empty_for_non_utf8() {
        assert!(substantive_check_lines(b"\xff\xfe").is_empty());
    }
}
