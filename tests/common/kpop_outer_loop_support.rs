use std::fs;
use std::path::Path;
use std::process::Command;

use super::{
    activate_test_home, command_output_with_timeout, seed_malvin_config, test_home_workspace,
    write_fake_kiss, cached_mock_executable, INTEGRATION_TEST_MALVIN_ARGS, MALVIN_TEST_CMD_TIMEOUT,
};

fn malvin_kpop_outer_cmd(
    root: &tempfile::TempDir,
    home: &Path,
    mock: &Path,
    extra_args: &[&str],
) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(root.path().join("workspace"))
        .env("HOME", home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", mock)
        .env("PATH", outer_loop_bin_path(root))
        .args(["kpop", "--max-hypotheses", "1"]);
    cmd.args(INTEGRATION_TEST_MALVIN_ARGS);
    cmd.args(extra_args).arg("investigate");
    cmd
}

fn outer_loop_bin_path(root: &tempfile::TempDir) -> String {
    let bin_dir = root.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_fake_kiss(&bin_dir.join("kiss"));
    format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

pub fn run_kpop_outer_loop(
    mock_js: &str,
    extra_args: &[&str],
    home_config_seed: Option<&str>,
) -> (std::process::Output, tempfile::TempDir) {
    let (root, home, workspace) = test_home_workspace();
    if let Some(content) = home_config_seed {
        activate_test_home(&home);
        seed_malvin_config(&workspace, content);
    }
    let mock = cached_mock_executable( mock_js);
    fs::write(workspace.join(".kissconfig"), "k = 1\n").expect("kissconfig");
    fs::write(workspace.join(".gitignore"), "baseline-gitignore\n").expect("gitignore");
    let mut cmd = malvin_kpop_outer_cmd(&root, &home, &mock, extra_args);
    let output = command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn");
    (output, root)
}

fn exp_log_file_name(path: &Path) -> Option<&str> {
    path.file_name()?.to_str()
}

fn is_gate_exp_log_path(path: &Path) -> bool {
    exp_log_file_name(path).is_some_and(|name| name.contains("_g"))
}

fn is_exp_log_md(path: &Path) -> bool {
    let Some(name) = exp_log_file_name(path) else {
        return false;
    };
    name.starts_with("exp_log_")
        && Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
}

pub fn gate_exp_logs_in_run(run_dir: &Path) -> Vec<std::path::PathBuf> {
    exp_logs_in_run(run_dir)
        .into_iter()
        .filter(|p| is_gate_exp_log_path(p))
        .collect()
}

pub fn exp_logs_in_run(run_dir: &Path) -> Vec<std::path::PathBuf> {
    let kpop_dir = run_dir.join("_kpop");
    let mut paths: Vec<_> = fs::read_dir(&kpop_dir)
        .expect("read _kpop")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| is_exp_log_md(p))
        .collect();
    paths.sort();
    paths
}

pub fn kpop_log_lines(stdout: &str) -> Vec<&str> {
    stdout.lines().filter(|line| line.contains("KPOP_LOG:")).collect()
}
