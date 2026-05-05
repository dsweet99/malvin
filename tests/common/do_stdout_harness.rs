#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
use super::{
    acp_mock_do_streaming_update_js, command_output_with_timeout, test_home_workspace,
    write_mock_executable, MALVIN_TEST_CMD_TIMEOUT,
};

#[cfg(unix)]
pub use malvin::config::DEFAULT_CLI_MODEL;

#[cfg(unix)]
pub const DO_WRAP_COLUMNS: &str = "32";

#[cfg(unix)]
pub fn run_do_with_named_mock_bin(
    mock_bin_name: &str,
    mock_js: &str,
    extra_args: &[&str],
    columns: Option<&str>,
) -> (std::process::Output, tempfile::TempDir, std::path::PathBuf) {
    let (root, home, workspace) = test_home_workspace();
    let mock = root.path().join(mock_bin_name);
    write_mock_executable(&mock, mock_js);
    let mut args = vec!["do"];
    args.extend_from_slice(extra_args);
    args.push("say hi");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock);
    if let Some(c) = columns {
        cmd.env("COLUMNS", c);
    }
    cmd.args(args);
    let out =
        command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin do");
    (out, root, workspace)
}

#[cfg(unix)]
pub fn run_do_with_mock_js(
    mock_js: &str,
    extra_args: &[&str],
    columns: Option<&str>,
) -> std::process::Output {
    let (out, root, _) =
        run_do_with_named_mock_bin("mock-agent-acp-do", mock_js, extra_args, columns);
    drop(root);
    out
}

#[cfg(unix)]
pub fn run_do_with_mock(extra_args: &[&str]) -> std::process::Output {
    run_do_with_mock_js(&acp_mock_do_streaming_update_js(), extra_args, None)
}

#[cfg(unix)]
pub fn run_do_with_columns_mock(mock_js: &str, extra_args: &[&str]) -> std::process::Output {
    run_do_with_mock_js(mock_js, extra_args, Some(DO_WRAP_COLUMNS))
}

#[cfg(unix)]
pub fn run_do_long_text_mock(extra_args: &[&str]) -> std::process::Output {
    use super::acp_mock_do_streaming_long_agent_msg_js;
    run_do_with_columns_mock(&acp_mock_do_streaming_long_agent_msg_js(), extra_args)
}

#[cfg(unix)]
pub fn run_do_wordy_long_mock(extra_args: &[&str]) -> std::process::Output {
    use super::acp_mock_do_streaming_wordy_long_msg_js;
    run_do_with_columns_mock(&acp_mock_do_streaming_wordy_long_msg_js(), extra_args)
}

#[cfg(unix)]
pub fn run_do_with_mock_and_argv(extra_args: &[&str]) -> (std::process::Output, Vec<String>) {
    let mut args: Vec<&str> = vec!["do"];
    args.extend_from_slice(extra_args);
    args.push("say hi");
    run_malvin_with_captured_argv(&args)
}

#[cfg(unix)]
pub struct MalvinCapturePaths<'a> {
    pub workspace: &'a Path,
    pub home: &'a Path,
    pub mock: &'a Path,
    pub capture: &'a Path,
}

#[cfg(unix)]
impl MalvinCapturePaths<'_> {
    pub fn malvin_command(self, malvin_args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
        cmd.current_dir(self.workspace)
            .env("HOME", self.home)
            .env("CURSOR_AGENT_API_KEY", "test-key")
            .env("MALVIN_AGENT_ACP_BIN", self.mock)
            .env("MALVIN_CAPTURE_ARGS_PATH", self.capture)
            .args(malvin_args);
        cmd
    }
}

#[cfg(unix)]
pub fn run_malvin_with_captured_argv(malvin_args: &[&str]) -> (std::process::Output, Vec<String>) {
    let (root, home, workspace) = test_home_workspace();
    let capture = root.path().join("captured-argv.txt");
    let mock = root.path().join("mock-agent-acp-do");
    write_mock_executable(&mock, &acp_mock_do_streaming_update_js());
    let cap = MalvinCapturePaths {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        capture: &capture,
    };
    let mut cmd = cap.malvin_command(malvin_args);
    let out = command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin");
    let captured_args = std::fs::read_to_string(&capture)
        .unwrap_or_default()
        .lines()
        .map(std::string::ToString::to_string)
        .collect();
    (out, captured_args)
}

#[cfg(unix)]
pub fn stdout_lines_preserve_shape(stdout: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(stdout)
        .split('\n')
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect()
}

#[cfg(unix)]
pub fn nonempty_stdout_lines(stdout: &[u8]) -> Vec<String> {
    stdout_lines_preserve_shape(stdout)
        .into_iter()
        .filter(|l| !l.is_empty())
        .collect()
}

#[cfg(unix)]
pub fn first_do_log_path(workspace: &std::path::Path) -> std::path::PathBuf {
    let sub = std::fs::read_dir(workspace.join("_malvin"))
        .expect("_malvin")
        .flatten()
        .find(|e| e.path().is_dir())
        .expect("run dir");
    sub.path().join("do.log")
}

#[cfg(unix)]
pub fn assert_stdout_has_no_chrome(lines: &[String]) {
    assert!(
        lines
            .iter()
            .all(|l| !l.contains("Command: ") && !l.contains("Logs: ") && !l.contains("TIMING: ")),
        "expected do stdout without startup/timing chrome, got {lines:?}"
    );
    assert!(
        lines.iter().all(|l| !l.contains("]: DONE")),
        "expected do stdout without DONE chrome, got {lines:?}"
    );
}
