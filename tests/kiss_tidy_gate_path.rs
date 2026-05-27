//! Tidy quality-gate and ACP ordering checks (split from `kiss_code_kpop_path` for kiss limits).

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    MALVIN_TEST_CMD_TIMEOUT, acp_mock_tidy_kpop_steps_js, command_output_with_timeout,
    seed_git_kiss_cargo_gate_workspace, seed_malvin_checks, test_home_workspace,
    write_failing_gate_tools,
    write_fake_kiss, write_mock_executable,
};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
fn seed_tidy_workspace(workspace: &Path) {
    seed_git_kiss_cargo_gate_workspace(workspace);
    std::fs::write(workspace.join("script.py"), "print('broken')\n").expect("write python file");
}

#[cfg(unix)]
struct MalvinTidySpawn<'a> {
    workspace: &'a Path,
    home: &'a Path,
    mock: &'a Path,
    path: &'a str,
    timeout: std::time::Duration,
}

#[cfg(unix)]
fn spawn_malvin_tidy(c: &MalvinTidySpawn<'_>) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(c.workspace)
        .env("HOME", c.home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", c.mock)
        .env("PATH", c.path)
        .args(["tidy", "--max-loops", "1"]);
    command_output_with_timeout(&mut cmd, c.timeout).expect("spawn malvin")
}

#[cfg(unix)]
fn set_mode755(path: &Path) {
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
struct TidySkipFixture {
    _root: tempfile::TempDir,
    workspace: std::path::PathBuf,
    home: std::path::PathBuf,
    mock: std::path::PathBuf,
    path: String,
}

#[cfg(unix)]
fn tidy_skip_agent_fixture() -> TidySkipFixture {
    let (root, home, workspace) = test_home_workspace();
    std::fs::create_dir(workspace.join(".git")).expect("mkdir .git");
    seed_malvin_checks(&workspace, "kiss check\n");
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_fake_kiss(&bin_dir.join("kiss"));
    let mock = root.path().join("mock-agent-must-not-run");
    std::fs::write(&mock, "#!/usr/bin/env sh\nexit 99\n").expect("write mock");
    set_mode755(&mock);
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    TidySkipFixture {
        _root: root,
        workspace,
        home,
        mock,
        path,
    }
}

#[cfg(unix)]
fn run_malvin_tidy_no_auth_keys(
    workspace: &Path,
    home: &Path,
    mock: &Path,
    path: &str,
) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(workspace)
        .env("HOME", home)
        .env_remove("CURSOR_AGENT_API_KEY")
        .env_remove("CURSOR_API_KEY")
        .env_remove("AGENT_API_KEY")
        .env("MALVIN_AGENT_ACP_BIN", mock)
        .env("PATH", path)
        .args(["tidy"]);
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
}

#[cfg_attr(unix, test)]
fn malvin_tidy_skips_agent_when_quality_gates_already_pass() {
    let fx = tidy_skip_agent_fixture();
    let out = run_malvin_tidy_no_auth_keys(&fx.workspace, &fx.home, &fx.mock, &fx.path);
    assert!(
        out.status.success(),
        "expected tidy to skip agent when gates pass; status={:?} stdout={} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("DONE"), "expected fast-path DONE; stdout={stdout:?}");
    assert!(
        !stdout.contains("[>kpop") && !stdout.contains("[<kpop"),
        "agent must not run when gates already pass; stdout={stdout:?}"
    );
}

#[cfg_attr(unix, test)]
fn malvin_tidy_runs_quality_gates_around_kpop_when_gates_fail() {
    let (root, home, workspace) = test_home_workspace();
    seed_tidy_workspace(&workspace);
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let trace = root.path().join("quality-trace.log");
    write_failing_gate_tools(&bin_dir, &trace);
    let mock = root.path().join("mock-agent-acp-tidy");
    write_mock_executable(&mock, &acp_mock_tidy_kpop_steps_js());
    let original_path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{original_path}", bin_dir.display());

    let out = spawn_malvin_tidy(&MalvinTidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path: &path,
        timeout: MALVIN_TEST_CMD_TIMEOUT + MALVIN_TEST_CMD_TIMEOUT,
    });

    assert!(
        !out.status.success(),
        "expected tidy to fail when post-ACP quality gates fail: {out:?}"
    );
    let trace_log = std::fs::read_to_string(&trace).unwrap_or_default();
    assert!(
        !trace_log.is_empty(),
        "expected quality gates to run around kpop: {trace_log}"
    );
    assert!(
        trace_log.contains("kiss"),
        "expected at least one quality gate command in trace: {trace_log}"
    );
}
