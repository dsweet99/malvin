#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::process::Command;
#[cfg(unix)]
use std::time::Instant;

#[cfg(unix)]
use super::child_wait::{spawn_piped_process_group, wait_child_with_timeout};
#[cfg(unix)]
use super::workspace::{test_home_workspace, write_fake_kiss, write_mock_executable};

#[cfg(unix)]
pub const MALVIN_TEST_CMD_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(12);

#[cfg(unix)]
pub struct GroundMockOpts {
    pub no_tee: bool,
    pub with_kissconfig: bool,
}

#[cfg(unix)]
fn fake_bin_with_kiss(root: &std::path::Path) -> std::path::PathBuf {
    let bin_dir = root.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_fake_kiss(&bin_dir.join("kiss"));
    bin_dir
}

#[cfg(unix)]
fn ground_agent_mock_path(root: &std::path::Path, mock_js: &str) -> std::path::PathBuf {
    let mock = root.join("mock-agent-acp-ground");
    write_mock_executable(&mock, mock_js);
    mock
}

#[cfg(unix)]
fn path_prefix_bin(bin_dir: &std::path::Path) -> String {
    format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

#[cfg(unix)]
pub fn run_ground_with_mock_js_with_setup<F>(
    mock_js: &str,
    opts: &GroundMockOpts,
    setup: F,
) -> (std::process::Output, tempfile::TempDir, PathBuf)
where
    F: FnOnce(&Path),
{
    let (root, home, workspace) = test_home_workspace();
    setup(&workspace);
    let bin_dir = fake_bin_with_kiss(root.path());
    let mock = ground_agent_mock_path(root.path(), mock_js);
    if opts.with_kissconfig {
        std::fs::write(workspace.join(".kissconfig"), "k\n").expect("write kissconfig");
    }
    let path = path_prefix_bin(&bin_dir);
    let mut args: Vec<&str> = vec!["ground"];
    if opts.no_tee {
        args.insert(0, "--no-tee");
    }
    let out = command_output_with_timeout(
        Command::new(env!("CARGO_BIN_EXE_malvin"))
            .current_dir(&workspace)
            .env("HOME", &home)
            .env("CURSOR_AGENT_API_KEY", "test-key")
            .env("MALVIN_AGENT_ACP_BIN", &mock)
            .env("PATH", path)
            .args(args),
        MALVIN_TEST_CMD_TIMEOUT,
    )
    .expect("spawn malvin ground");
    (out, root, workspace)
}

#[cfg(unix)]
pub fn run_ground_with_mock_js(
    mock_js: &str,
    opts: &GroundMockOpts,
) -> (std::process::Output, tempfile::TempDir, PathBuf) {
    run_ground_with_mock_js_with_setup(mock_js, opts, |_| {})
}

#[cfg(unix)]
pub fn command_output_with_timeout(
    cmd: &mut Command,
    timeout: std::time::Duration,
) -> std::io::Result<std::process::Output> {
    let (child, stdout_jh, stderr_jh) = spawn_piped_process_group(cmd)?;
    wait_child_with_timeout(
        child,
        stdout_jh,
        stderr_jh,
        Instant::now() + timeout,
    )
}
