#[cfg(all(unix, target_os = "linux"))]
use std::path::PathBuf;
#[cfg(all(unix, target_os = "linux"))]
use std::process::Command;

#[cfg(all(unix, target_os = "linux"))]
fn kpop_multiturn_prep(mock_js: &str) -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf, String) {
    use super::{seed_malvin_checks, test_home_workspace, write_fake_kiss, cached_mock_executable};

    let (root, home, workspace) = test_home_workspace();
    seed_malvin_checks(&workspace, "kiss check\n");
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let mock = cached_mock_executable( mock_js);
    write_fake_kiss(&bin_dir.join("kiss"));
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    (root, home, workspace, mock, path)
}

#[cfg(all(unix, target_os = "linux"))]
pub fn run_kpop_multiturn_investigate(
    mock_js: &str,
) -> (std::process::Output, tempfile::TempDir, std::path::PathBuf) {
    use super::{
        INTEGRATION_TEST_MALVIN_ARGS, MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout,
    };

    let (root, home, workspace, mock, path) = kpop_multiturn_prep(mock_js);
    std::fs::write(workspace.join(".kissconfig"), "k = 1\n").expect("write kissconfig");
    std::fs::write(workspace.join(".gitignore"), "g = 1\n").expect("write gitignore");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .env("PATH", path)
        .arg("kpop");
    cmd.args(INTEGRATION_TEST_MALVIN_ARGS);
    cmd.args([
        "--max-loops",
        "0",
                "investigate",
    ]);
    let out =
        command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin kpop");
    (out, root, workspace)
}
