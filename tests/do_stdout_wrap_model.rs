mod common;

#[cfg(unix)]
use common::{
    DEFAULT_CLI_MODEL, DO_WRAP_COLUMNS, assert_stdout_has_no_chrome, nonempty_stdout_lines,
    run_do_long_text_mock, run_do_with_mock, run_do_with_mock_and_argv, run_do_wordy_long_mock,
    run_malvin_with_captured_argv, stdout_lines_preserve_shape,
};

#[cfg_attr(unix, test)]
fn do_wraps_long_raw_agent_line_when_columns_set() {
    let out = run_do_long_text_mock(&[]);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text_lines = nonempty_stdout_lines(&out.stdout);
    let wrap_lines: Vec<&str> = text_lines
        .iter()
        .map(String::as_str)
        .filter(|l| l.chars().all(|c| c == 'a'))
        .collect();
    assert!(
        wrap_lines.len() > 1,
        "expected word-wrapped stdout (multiple non-empty lines), got {text_lines:?}"
    );
    let joined: String = wrap_lines.iter().copied().collect();
    assert_eq!(joined.len(), 120);
    assert!(joined.chars().all(|c| c == 'a'));
    let col_n: usize = DO_WRAP_COLUMNS.parse().expect("columns");
    assert!(
        wrap_lines.iter().all(|l| l.chars().count() <= col_n),
        "each wrapped segment should fit COLUMNS: {wrap_lines:?}"
    );
    assert!(!String::from_utf8_lossy(&out.stdout).contains("jsonrpc"));
}

#[cfg_attr(unix, test)]
fn do_stdout_includes_thoughts_only_with_flag() {
    let out = run_do_with_mock(&["--thoughts"]);
    assert!(out.status.success(), "malvin do --thoughts failed: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines = stdout_lines_preserve_shape(&out.stdout);
    assert!(
        lines.iter().all(|l| !l.contains("do...")),
        "did not expect do stdout chrome, got: {lines:?}"
    );
    assert!(lines.iter().any(|l| l == "agent message"), "got: {lines:?}");
    assert!(
        lines.iter().any(|l| l == "hidden thought"),
        "stdout was {stdout:?}"
    );
    assert!(
        !stdout.contains("[hidden thought]"),
        "stdout was {stdout:?}"
    );
    assert!(
        lines.iter().all(|l| !l.contains(":[<do")),
        "did not expect tagged do stdout lines, got: {lines:?}"
    );
    assert_stdout_has_no_chrome(&lines);
    assert!(stdout.contains("hidden thought"), "stdout was {stdout:?}");
    assert!(!stdout.contains("\"jsonrpc\""), "stdout was {stdout:?}");
}

#[cfg_attr(unix, test)]
fn do_forwards_default_model_and_force_to_agent() {
    let (out, argv) = run_do_with_mock_and_argv(&[]);
    assert!(out.status.success(), "malvin do failed: {out:?}");
    let model_values: Vec<&str> = argv
        .windows(2)
        .filter(|w| w[0] == "--model")
        .map(|w| w[1].as_str())
        .collect();
    assert!(
        model_values == vec![DEFAULT_CLI_MODEL],
        "expected exactly one forwarded --model {DEFAULT_CLI_MODEL}; argv={argv:?}"
    );
    let force_count = argv.iter().filter(|arg| arg.as_str() == "--force").count();
    assert!(
        force_count == 1,
        "expected exactly one forwarded --force by default; argv={argv:?}"
    );
    assert!(
        !argv.iter().any(|arg| arg == "--no-force"),
        "did not expect forwarded --no-force; argv={argv:?}"
    );
}

#[cfg_attr(unix, test)]
fn do_respects_no_force_and_explicit_model_flags() {
    let (out, argv) =
        run_malvin_with_captured_argv(&["--no-force", "--model", "composer-x", "do", "say hi"]);
    assert!(out.status.success(), "malvin do failed: {out:?}");
    let model_values: Vec<&str> = argv
        .windows(2)
        .filter(|w| w[0] == "--model")
        .map(|w| w[1].as_str())
        .collect();
    assert!(
        model_values == vec!["composer-x"],
        "expected exactly one forwarded --model composer-x; argv={argv:?}"
    );
    assert!(
        !argv.iter().any(|arg| arg == "--force"),
        "did not expect --force with --no-force; argv={argv:?}"
    );
    assert!(
        !argv.iter().any(|arg| arg == "--no-force"),
        "did not expect forwarded --no-force; argv={argv:?}"
    );
}

#[cfg_attr(unix, test)]
fn do_wraps_wordy_long_text_at_word_boundaries() {
    let out = run_do_wordy_long_mock(&[]);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text_lines = nonempty_stdout_lines(&out.stdout);
    let wrap_lines: Vec<&str> = text_lines
        .iter()
        .map(String::as_str)
        .filter(|l| l.split_whitespace().all(|w| w == "abcdefghij"))
        .collect();
    assert!(
        wrap_lines.len() > 1,
        "expected word-wrapped stdout, got {text_lines:?}"
    );
    let col_n: usize = DO_WRAP_COLUMNS.parse().expect("columns");
    assert!(
        wrap_lines.iter().all(|l| l.chars().count() <= col_n),
        "each wrapped line should fit COLUMNS={col_n}: {wrap_lines:?}"
    );
    let expected_word = "abcdefghij";
    for line in &wrap_lines {
        for word in line.split_whitespace() {
            assert!(
                word == expected_word,
                "word-wrap should not split words; found partial word {word:?} in line {line:?}"
            );
        }
    }
}
