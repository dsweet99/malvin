#[cfg(all(unix, target_os = "linux"))]
use std::os::unix::fs::PermissionsExt;
#[cfg(all(unix, target_os = "linux"))]
use std::path::{Path, PathBuf};
#[cfg(all(unix, target_os = "linux"))]
use std::process::Command;

#[cfg(all(unix, target_os = "linux"))]
pub struct PtyRun {
    pub _root: tempfile::TempDir,
    pub home: PathBuf,
    pub workspace: PathBuf,
    pub output: std::process::Output,
}

#[cfg(all(unix, target_os = "linux"))]
fn chmod755(path: &Path) {
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(all(unix, target_os = "linux"))]
pub struct PtyEnv {
    pub root: tempfile::TempDir,
    pub home: PathBuf,
    pub workspace: PathBuf,
    pub bin_dir: PathBuf,
    pub mock: PathBuf,
}

#[cfg(all(unix, target_os = "linux"))]
fn pty_malvin_workspace(mock_js: &str) -> PtyEnv {
    use super::{test_home_workspace, write_fake_kiss, write_mock_executable};

    let (root, home, workspace) = test_home_workspace();
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let mock = root.path().join("mock-agent-acp-md");
    write_mock_executable(&mock, mock_js);
    write_fake_kiss(&bin_dir.join("kiss"));
    PtyEnv {
        root,
        home,
        workspace,
        bin_dir,
        mock,
    }
}

#[cfg(all(unix, target_os = "linux"))]
fn pty_write_malvin_runner_script(
    env: &PtyEnv,
    malvin_args_line: &str,
    columns: Option<&str>,
) -> PathBuf {
    let malvin = env!("CARGO_BIN_EXE_malvin");
    let sh = env.root.path().join("run-malvin.sh");
    let columns_export = columns
        .map(|value| format!("export COLUMNS=\"{value}\"\n"))
        .unwrap_or_default();
    let body = format!(
        "#!/bin/sh\nunset NO_COLOR\nexport PATH=\"{}:$PATH\"\nexport HOME=\"{}\"\nexport CURSOR_AGENT_API_KEY=test\nexport MALVIN_AGENT_ACP_BIN=\"{}\"\n{}cd \"{}\"\nexec \"{}\" {}\n",
        env.bin_dir.display(),
        env.home.display(),
        env.mock.display(),
        columns_export,
        env.workspace.display(),
        malvin,
        malvin_args_line
    );
    std::fs::write(&sh, body).expect("write run-malvin.sh");
    chmod755(&sh);
    sh
}

#[cfg(all(unix, target_os = "linux"))]
pub fn run_malvin_under_script_with_mock(
    mock_js: &str,
    malvin_args_line: &str,
    columns: Option<&str>,
) -> PtyRun {
    use super::{MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout};

    let env = pty_malvin_workspace(mock_js);
    let sh = pty_write_malvin_runner_script(&env, malvin_args_line, columns);
    let mut cmd = Command::new("script");
    cmd.args([
        "-q",
        "-e",
        "-c",
        sh.to_str().expect("run-malvin.sh utf8"),
        "/dev/null",
    ]);
    cmd.stdin(std::process::Stdio::null());
    let output =
        command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("script malvin");
    PtyRun {
        _root: env.root,
        home: env.home,
        workspace: env.workspace,
        output,
    }
}

#[cfg(all(unix, target_os = "linux"))]
pub fn run_malvin_under_script(malvin_args_line: &str) -> std::process::Output {
    use super::acp_mock_code_streaming_bold_markdown_js;
    run_malvin_under_script_with_mock(
        &acp_mock_code_streaming_bold_markdown_js(),
        malvin_args_line,
        None,
    )
    .output
}

#[cfg(all(unix, target_os = "linux"))]
pub fn run_code_max_loops_zero_under_script(extra_args: &[&str]) -> std::process::Output {
    let mut args_line = String::from("code --trust-the-plan --max-loops 0 ship");
    for a in extra_args {
        args_line.push(' ');
        args_line.push_str(a);
    }
    run_malvin_under_script(&args_line)
}

#[cfg(all(unix, target_os = "linux"))]
pub fn run_kpop_bold_markdown_under_script(extra_args: &[&str]) -> std::process::Output {
    let mut args_line = String::from("kpop --max-loops 1 --max-hypotheses 50 investigate");
    for a in extra_args {
        args_line.push(' ');
        args_line.push_str(a);
    }
    run_malvin_under_script(&args_line)
}

#[cfg(all(unix, target_os = "linux"))]
pub fn run_do_under_script(global_lead: &[&str]) -> std::process::Output {
    let mut args_line = global_lead.join(" ");
    if !args_line.is_empty() {
        args_line.push(' ');
    }
    args_line.push_str("do \"say hi\"");
    run_malvin_under_script(&args_line)
}

#[cfg(all(unix, target_os = "linux"))]
pub fn read_all_logs(run_dir: &Path) -> String {
    let mut paths: Vec<PathBuf> = std::fs::read_dir(run_dir)
        .expect("read run dir")
        .map(|entry| entry.expect("dir entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "log"))
        .collect();
    paths.sort();
    let chunks: Vec<String> = paths
        .into_iter()
        .map(|path| std::fs::read_to_string(path).expect("read log"))
        .collect();
    chunks.join("\n")
}

#[cfg(all(unix, target_os = "linux"))]
pub fn assert_markdown_stdout_and_logs(run: &PtyRun) {
    let stdout = String::from_utf8_lossy(&run.output.stdout);
    assert!(
        !stdout.contains("# md-heading-xyz"),
        "expected stdout markdown rendering to consume heading markers: {stdout:?}"
    );
    assert!(
        stdout.contains("md-item-xyz"),
        "expected stdout markdown rendering to keep list item text visible: {stdout:?}"
    );
    assert!(
        !stdout.contains("**md-bold-xyz**"),
        "expected styled stdout to consume bold markers: {stdout:?}"
    );
    assert!(
        stdout.contains("\x1b[1m"),
        "expected bold ANSI on TTY stdout: {stdout:?}"
    );
    assert!(
        !stdout.contains("\"jsonrpc\""),
        "stdout leaked JSON-RPC protocol lines: {stdout:?}"
    );
    let run_dir = super::only_run_dir(&run.workspace, &run.home);
    let logs = read_all_logs(&run_dir);
    assert!(
        logs.contains("# md-heading-xyz"),
        "expected raw heading markdown in logs: {logs:?}"
    );
    assert!(
        logs.contains("- md-item-xyz"),
        "expected raw list markdown in logs: {logs:?}"
    );
    assert!(
        logs.contains("**md-bold-xyz**"),
        "expected raw bold markdown in logs: {logs:?}"
    );
    assert!(
        !logs.contains("\x1b[1m"),
        "run logs must stay raw without ANSI styling: {logs:?}"
    );
}
