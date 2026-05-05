//! `malvin code` fails fast when `kiss` is not on `PATH`.

#[cfg(unix)]
mod common;

use std::process::Command;

#[cfg(unix)]
use common::{
    MALVIN_TEST_CMD_TIMEOUT, acp_mock_code_streaming_update_js, command_output_with_timeout,
    seed_git_kiss_cargo_gate_workspace, test_home_workspace, write_failing_gate_tools,
    write_mock_executable,
};
#[cfg(unix)]
use std::path::Path;

#[cfg(unix)]
fn clear_agent_api_env(cmd: &mut Command) {
    cmd.env_remove("CURSOR_AGENT_API_KEY")
        .env_remove("CURSOR_API_KEY")
        .env_remove("AGENT_API_KEY")
        .env_remove("MALVIN_AGENT_ACP_BIN");
}

#[cfg(unix)]
fn run_malvin_path_timed(
    path_bin: &std::path::Path,
    configure: impl FnOnce(&mut Command),
) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.env("PATH", path_bin);
    configure(&mut cmd);
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
}

fn assert_malvin_subcommand_fails_without_kiss(args: &[&str]) {
    let path_root = tempfile::tempdir().unwrap();
    let isolated_bin = path_root.path().join("bin");
    std::fs::create_dir_all(&isolated_bin).unwrap();

    #[cfg(unix)]
    let out = run_malvin_path_timed(&isolated_bin, |c| {
        c.args(args);
    });
    #[cfg(not(unix))]
    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .env("PATH", &isolated_bin)
        .args(args)
        .output()
        .expect("spawn malvin");

    assert!(
        !out.status.success(),
        "expected non-zero exit; stdout/stderr: {out:?}"
    );
    let msg = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        msg.contains("kiss") && msg.contains("cargo install kiss-ai"),
        "expected kiss + install hint; got: {msg:?}"
    );
}

fn assert_malvin_subcommand_not_kiss_gated_without_auth(args: &[&str]) {
    let path_root = tempfile::tempdir().unwrap();
    let isolated_bin = path_root.path().join("bin");
    std::fs::create_dir_all(&isolated_bin).unwrap();
    #[cfg(unix)]
    let out = run_malvin_path_timed(&isolated_bin, |c| {
        clear_agent_api_env(c);
        c.args(args);
    });
    #[cfg(not(unix))]
    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .env("PATH", &isolated_bin)
        .env_remove("CURSOR_AGENT_API_KEY")
        .env_remove("CURSOR_API_KEY")
        .env_remove("AGENT_API_KEY")
        .env_remove("MALVIN_AGENT_ACP_BIN")
        .args(args)
        .output()
        .expect("spawn malvin");
    assert!(
        !out.status.success(),
        "expected non-zero exit; stdout/stderr: {out:?}"
    );
    let msg = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        msg.contains("not authenticated") && msg.contains("CURSOR_AGENT_API_KEY"),
        "expected auth failure path (not kiss precheck); got: {msg:?}"
    );
    assert!(
        !msg.contains("cargo install kiss-ai")
            && !msg.contains("`kiss` is not installed or not on PATH"),
        "expected auth failure path for no-kiss-gate subcommand; got: {msg:?}"
    );
}

#[test]
fn malvin_code_fails_fast_when_kiss_missing_from_path() {
    assert_malvin_subcommand_fails_without_kiss(&["code", "x"]);
}

#[test]
fn malvin_tidy_fails_fast_when_kiss_missing_from_path() {
    assert_malvin_subcommand_fails_without_kiss(&["tidy"]);
}

#[test]
fn malvin_tidy_kiss_missing_error_cites_tidy_subcommand() {
    let path_root = tempfile::tempdir().unwrap();
    let isolated_bin = path_root.path().join("bin");
    std::fs::create_dir_all(&isolated_bin).unwrap();
    #[cfg(unix)]
    let out = run_malvin_path_timed(&isolated_bin, |c| {
        c.arg("tidy");
    });
    #[cfg(not(unix))]
    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .env("PATH", &isolated_bin)
        .arg("tidy")
        .output()
        .expect("spawn malvin");
    assert!(!out.status.success());
    let msg = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        msg.contains("`malvin tidy`"),
        "expected error to name the tidy subcommand; got: {msg:?}"
    );
    assert!(
        !msg.contains("`malvin code`"),
        "expected tidy path not to reuse code subcommand text; got: {msg:?}"
    );
}

#[test]
fn malvin_plan_fails_fast_when_kiss_missing_from_path() {
    assert_malvin_subcommand_fails_without_kiss(&["plan"]);
}

#[cfg(unix)]
fn seed_tidy_workspace(workspace: &Path) {
    seed_git_kiss_cargo_gate_workspace(workspace);
    std::fs::write(workspace.join("script.py"), "print('broken')\n").expect("write python file");
}

#[cfg(unix)]
fn spawn_malvin_tidy_with_mock_path(
    workspace: &Path,
    home: &Path,
    mock: &Path,
    path: &str,
) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(workspace)
        .env("HOME", home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", mock)
        .env("PATH", path)
        .args(["tidy", "--no-learn"]);
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
}

#[cfg_attr(unix, test)]
fn malvin_tidy_runs_quality_gates_after_acp() {
    let (root, home, workspace) = test_home_workspace();
    seed_tidy_workspace(&workspace);
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let trace = root.path().join("quality-trace.log");
    write_failing_gate_tools(&bin_dir, &trace);
    let mock = root.path().join("mock-agent-acp-tidy");
    write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
    let original_path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{original_path}", bin_dir.display());

    let out = spawn_malvin_tidy_with_mock_path(&workspace, &home, &mock, &path);

    assert!(
        !out.status.success(),
        "expected tidy to fail when post-ACP quality gates fail: {out:?}"
    );
    let trace_log = std::fs::read_to_string(&trace).unwrap_or_default();
    assert!(
        !trace_log.is_empty(),
        "expected quality gates to run after ACP: {trace_log}"
    );
    assert!(
        trace_log.contains("kiss"),
        "expected at least one post-ACP quality gate command to run: {trace_log}"
    );
}

#[test]
fn malvin_do_is_not_kiss_gated_when_kiss_missing_from_path() {
    assert_malvin_subcommand_not_kiss_gated_without_auth(&["do", "hello"]);
}

#[test]
fn malvin_kpop_is_not_kiss_gated_when_kiss_missing_from_path() {
    assert_malvin_subcommand_not_kiss_gated_without_auth(&["kpop", "x"]);
}
