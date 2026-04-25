mod common;

#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
use common::{
    acp_mock_do_streaming_long_agent_msg_js, acp_mock_do_streaming_update_js,
    acp_mock_do_streaming_wordy_long_msg_js, test_home_workspace, write_mock_executable,
};
#[cfg(unix)]
use malvin::config::DEFAULT_CLI_MODEL;

#[cfg(unix)]
const DO_WRAP_COLUMNS: &str = "32";

#[cfg(unix)]
fn run_do_with_mock(extra_args: &[&str]) -> std::process::Output {
    let (root, home, workspace) = test_home_workspace();
    let mock = root.path().join("mock-agent-acp-do");
    write_mock_executable(&mock, &acp_mock_do_streaming_update_js());
    let mut args = vec!["do"];
    args.extend_from_slice(extra_args);
    args.push("say hi");
    Command::new(env!("CARGO_BIN_EXE_malvin"))
        .current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .args(args)
        .output()
        .expect("spawn malvin do")
}

#[cfg(unix)]
fn run_do_with_columns_mock(mock_js: &str, extra_args: &[&str]) -> std::process::Output {
    let (root, home, workspace) = test_home_workspace();
    let mock = root.path().join("mock-agent-acp-do");
    write_mock_executable(&mock, mock_js);
    let mut args = vec!["do"];
    args.extend_from_slice(extra_args);
    args.push("say hi");
    Command::new(env!("CARGO_BIN_EXE_malvin"))
        .current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .env("COLUMNS", DO_WRAP_COLUMNS)
        .args(args)
        .output()
        .expect("spawn malvin do")
}

#[cfg(unix)]
fn run_do_long_text_mock(extra_args: &[&str]) -> std::process::Output {
    run_do_with_columns_mock(&acp_mock_do_streaming_long_agent_msg_js(), extra_args)
}

#[cfg(unix)]
fn run_do_wordy_long_mock(extra_args: &[&str]) -> std::process::Output {
    run_do_with_columns_mock(&acp_mock_do_streaming_wordy_long_msg_js(), extra_args)
}

#[cfg(unix)]
fn run_do_with_mock_and_argv(extra_args: &[&str]) -> (std::process::Output, Vec<String>) {
    let mut args: Vec<&str> = vec!["do"];
    args.extend_from_slice(extra_args);
    args.push("say hi");
    run_malvin_with_captured_argv(&args)
}

#[cfg(unix)]
fn run_malvin_with_captured_argv(malvin_args: &[&str]) -> (std::process::Output, Vec<String>) {
    let (root, home, workspace) = test_home_workspace();
    let capture = root.path().join("captured-argv.txt");
    let mock = root.path().join("mock-agent-acp-do");
    write_mock_executable(&mock, &acp_mock_do_streaming_update_js());
    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .env("MALVIN_CAPTURE_ARGS_PATH", &capture)
        .args(malvin_args)
        .output()
        .expect("spawn malvin");
    let captured_args = std::fs::read_to_string(&capture)
        .unwrap_or_default()
        .lines()
        .map(std::string::ToString::to_string)
        .collect();
    (out, captured_args)
}

#[cfg(unix)]
fn stdout_lines_preserve_shape(stdout: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(stdout)
        .split('\n')
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect()
}

#[cfg(unix)]
#[test]
fn do_stdout_shows_plain_output_without_jsonrpc_lines() {
    let out = run_do_with_mock(&[]);
    assert!(out.status.success(), "malvin do failed: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout_lines_preserve_shape(&out.stdout),
        vec!["agent message", ""]
    );
    assert!(!stdout.contains("hidden thought"), "stdout was {stdout:?}");
    assert!(!stdout.contains("\"jsonrpc\""), "stdout was {stdout:?}");
}

#[cfg(unix)]
#[test]
fn do_wraps_long_raw_agent_line_when_columns_set() {
    let out = run_do_long_text_mock(&[]);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout_s = String::from_utf8_lossy(&out.stdout);
    let text_lines: Vec<&str> = stdout_s
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .filter(|l| !l.is_empty())
        .collect();
    assert!(
        text_lines.len() > 1,
        "expected word-wrapped stdout (multiple non-empty lines), got {:?}",
        String::from_utf8_lossy(&out.stdout)
    );
    let joined: String = text_lines.iter().copied().collect();
    assert_eq!(joined.len(), 120);
    assert!(joined.chars().all(|c| c == 'a'));
    let col_n: usize = DO_WRAP_COLUMNS.parse().expect("columns");
    assert!(
        text_lines.iter().all(|l| l.chars().count() <= col_n),
        "each wrapped segment should fit COLUMNS: {text_lines:?}"
    );
    assert!(!String::from_utf8_lossy(&out.stdout).contains("jsonrpc"));
}

#[cfg(unix)]
#[test]
fn do_stdout_includes_thoughts_only_with_flag() {
    let out = run_do_with_mock(&["--thoughts"]);
    assert!(out.status.success(), "malvin do --thoughts failed: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines = stdout_lines_preserve_shape(&out.stdout);
    assert_eq!(lines.len(), 4, "unexpected stdout shape: {lines:?}");
    assert!(lines[0].contains("do..."), "expected outgoing do bracket line, got: {lines:?}");
    assert_eq!(lines[1], "agent message");
    assert!(lines[2].contains("hidden thought"), "stdout was {stdout:?}");
    assert_eq!(lines[3], "");
    assert!(stdout.contains("hidden thought"), "stdout was {stdout:?}");
    assert!(!stdout.contains("\"jsonrpc\""), "stdout was {stdout:?}");
}

#[cfg(unix)]
#[test]
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

#[cfg(unix)]
#[test]
fn do_respects_no_force_and_explicit_model_flags() {
    let (out, argv) = run_malvin_with_captured_argv(&[
        "--no-force",
        "--model",
        "composer-x",
        "do",
        "say hi",
    ]);
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

#[cfg(unix)]
#[test]
fn do_wraps_wordy_long_text_at_word_boundaries() {
    let out = run_do_wordy_long_mock(&[]);
    assert!(
        out.status.success(),
        "malvin do failed: {:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout_s = String::from_utf8_lossy(&out.stdout);
    let text_lines: Vec<&str> = stdout_s
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .filter(|l| !l.is_empty())
        .collect();
    assert!(
        text_lines.len() > 1,
        "expected word-wrapped stdout, got {stdout_s:?}"
    );
    let col_n: usize = DO_WRAP_COLUMNS.parse().expect("columns");
    assert!(
        text_lines.iter().all(|l| l.chars().count() <= col_n),
        "each wrapped line should fit COLUMNS={col_n}: {text_lines:?}"
    );
    let expected_word = "abcdefghij";
    for line in &text_lines {
        for word in line.split_whitespace() {
            assert!(
                word == expected_word,
                "word-wrap should not split words; found partial word {word:?} in line {line:?}"
            );
        }
    }
}
