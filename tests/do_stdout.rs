mod common;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

#[cfg(unix)]
use common::{
    ACP_MOCK_INTEGRATION_DO_STREAMING_LONG_AGENT_MSG_JS, ACP_MOCK_INTEGRATION_DO_STREAMING_UPDATE_JS,
};
#[cfg(unix)]
use malvin::config::DEFAULT_CLI_MODEL;

#[cfg(unix)]
const DO_WRAP_COLUMNS: &str = "32";

#[cfg(unix)]
fn do_test_home_workspace() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let root = tempfile::tempdir().expect("tempdir");
    let home = root.path().join("home");
    let workspace = root.path().join("workspace");
    std::fs::create_dir_all(&home).expect("mkdir home");
    std::fs::create_dir_all(&workspace).expect("mkdir workspace");
    std::fs::write(workspace.join("grounding.md"), "x").expect("grounding");
    (root, home, workspace)
}

#[cfg(unix)]
fn write_mock_executable(path: &std::path::Path) {
    let script = format!("#!/usr/bin/env node\n{ACP_MOCK_INTEGRATION_DO_STREAMING_UPDATE_JS}");
    std::fs::write(path, script).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
fn write_long_agent_mock(path: &Path) {
    let script = format!("#!/usr/bin/env node\n{ACP_MOCK_INTEGRATION_DO_STREAMING_LONG_AGENT_MSG_JS}");
    std::fs::write(path, script).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
fn run_do_with_mock(extra_args: &[&str]) -> std::process::Output {
    let (root, home, workspace) = do_test_home_workspace();
    let mock = root.path().join("mock-agent-acp-do");
    write_mock_executable(&mock);
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
fn run_do_long_text_mock(extra_args: &[&str]) -> std::process::Output {
    let (root, home, workspace) = do_test_home_workspace();
    let mock = root.path().join("mock-agent-acp-do");
    write_long_agent_mock(&mock);
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
fn run_do_with_mock_and_argv(extra_args: &[&str]) -> (std::process::Output, Vec<String>) {
    let mut args: Vec<&str> = vec!["do"];
    args.extend_from_slice(extra_args);
    args.push("say hi");
    run_malvin_with_captured_argv(&args)
}

#[cfg(unix)]
fn run_malvin_with_captured_argv(malvin_args: &[&str]) -> (std::process::Output, Vec<String>) {
    let (root, home, workspace) = do_test_home_workspace();
    let capture = root.path().join("captured-argv.txt");
    let mock = root.path().join("mock-agent-acp-do");
    write_mock_executable(&mock);
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
