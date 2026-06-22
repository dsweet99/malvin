pub(super) fn log_contains_command(log: &str, expected: &str) -> bool {
    log.split('\n').any(|line| {
        line.split_whitespace()
            .collect::<Vec<_>>()
            .windows(expected.split_whitespace().count())
            .any(|window| window.join(" ") == expected)
    })
}

#[cfg(test)]
mod log_contains_command_tests {
    use super::log_contains_command;

    #[test]
    fn log_contains_command_matches_whitespace_normalized_tokens() {
        let log = "kiss check\nruff check\n";
        assert!(log_contains_command(log, "kiss check"));
        assert!(!log_contains_command(log, "pytest -sv tests"));
    }
}
