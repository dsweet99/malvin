#[cfg(all(unix, target_os = "linux"))]
use std::path::{Path, PathBuf};

#[cfg(all(unix, target_os = "linux"))]
pub use super::cli_parity_tty_openpty::PtyEnv;

#[cfg(all(unix, target_os = "linux"))]
pub struct PtyRun {
    pub _root: tempfile::TempDir,
    pub home: PathBuf,
    pub workspace: PathBuf,
    pub output: std::process::Output,
}

#[cfg(all(unix, target_os = "linux"))]
use super::cli_parity_tty_openpty;

#[cfg(all(unix, target_os = "linux"))]
fn pty_malvin_workspace(mock_js: &str) -> PtyEnv {
    use super::{cached_mock_executable, test_home_workspace, write_fake_kiss};

    let (root, home, workspace) = test_home_workspace();
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let mock = cached_mock_executable(mock_js);
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
pub fn run_malvin_under_openpty_with_mock(
    mock_js: &str,
    malvin_args_line: &str,
    columns: Option<&str>,
) -> PtyRun {
    let env = pty_malvin_workspace(mock_js);
    let output = cli_parity_tty_openpty::run_malvin_under_openpty(
        &env,
        malvin_args_line,
        columns,
    );
    PtyRun {
        _root: env.root,
        home: env.home,
        workspace: env.workspace,
        output,
    }
}

#[cfg(all(unix, target_os = "linux"))]
pub fn run_malvin_under_openpty(malvin_args_line: &str) -> std::process::Output {
    use super::acp_mock_code_streaming_bold_markdown_js;
    run_malvin_under_openpty_with_mock(
        &acp_mock_code_streaming_bold_markdown_js(),
        malvin_args_line,
        None,
    )
    .output
}

#[cfg(all(unix, target_os = "linux"))]
pub fn run_code_max_loops_zero_under_openpty(extra_args: &[&str]) -> std::process::Output {
    let mut args_line = String::from("code --trust-the-plan --max-loops 0 ship");
    for a in extra_args {
        args_line.push(' ');
        args_line.push_str(a);
    }
    run_malvin_under_openpty(&args_line)
}

#[cfg(all(unix, target_os = "linux"))]
pub fn run_kpop_bold_markdown_under_openpty(extra_args: &[&str]) -> std::process::Output {
    let mut args_line = String::from("kpop --max-loops 1 --max-hypotheses 1 investigate");
    for a in extra_args {
        args_line.push(' ');
        args_line.push_str(a);
    }
    run_malvin_under_openpty(&args_line)
}

#[cfg(all(unix, target_os = "linux"))]
pub fn run_do_under_openpty(global_lead: &[&str]) -> std::process::Output {
    let mut args_line = global_lead.join(" ");
    if !args_line.is_empty() {
        args_line.push(' ');
    }
    args_line.push_str("do \"say hi\"");
    run_malvin_under_openpty(&args_line)
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
