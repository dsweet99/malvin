pub(super) fn log_contains_command(log: &str, expected: &str) -> bool {
    log.split('\n').any(|line| {
        line.split_whitespace()
            .collect::<Vec<_>>()
            .windows(expected.split_whitespace().count())
            .any(|window| window.join(" ") == expected)
    })
}
