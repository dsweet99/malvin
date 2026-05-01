//! `malvin code` fails fast when `kiss` is not on `PATH`.

#[cfg(unix)]
mod common;

use std::process::Command;

#[cfg(unix)]
use common::{
    MALVIN_TEST_CMD_TIMEOUT, acp_mock_code_streaming_update_js, command_output_with_timeout,
    test_home_workspace, write_mock_executable,
};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::path::Path;

fn assert_malvin_subcommand_fails_without_kiss(args: &[&str]) {
    let path_root = tempfile::tempdir().unwrap();
    let isolated_bin = path_root.path().join("bin");
    std::fs::create_dir_all(&isolated_bin).unwrap();

    #[cfg(unix)]
    let out = {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
        cmd.env("PATH", &isolated_bin).args(args);
        command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
    };
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
    let out = {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
        cmd.env("PATH", &isolated_bin)
            .env_remove("CURSOR_AGENT_API_KEY")
            .env_remove("CURSOR_API_KEY")
            .env_remove("AGENT_API_KEY")
            .env_remove("MALVIN_AGENT_ACP_BIN")
            .args(args);
        command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
    };
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

#[cfg(unix)]
fn write_failing_command(path: &Path, trace: &Path) {
    let name = path.file_name().unwrap().to_string_lossy();
    std::fs::write(
        path,
        format!(
            "#!/usr/bin/env sh\necho \"{name} $@\" >> \"{}\"\nexit 1\n",
            trace.display()
        ),
    )
    .expect("write failing command");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
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
#[cfg(unix)]
fn malvin_tidy_runs_quality_gates_after_acp() {
    let (root, home, workspace) = test_home_workspace();
    std::fs::create_dir(workspace.join(".git")).expect("mkdir git marker");
    std::fs::write(
        workspace.join(".kissconfig"),
        "[gate]\ntest_coverage_threshold = 90\n",
    )
    .expect("write kissconfig");
    std::fs::write(
        workspace.join("Cargo.toml"),
        "[package]\nname = 'm'\nversion = '0.1.0'\n",
    )
    .expect("write cargo manifest");
    std::fs::write(workspace.join("script.py"), "print('broken')\n").expect("write python file");

    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let trace = root.path().join("quality-trace.log");
    for name in ["kiss", "cargo", "ruff", "pytest"] {
        write_failing_command(&bin_dir.join(name), &trace);
    }
    let mock = root.path().join("mock-agent-acp-tidy");
    write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
    let original_path = std::env::var("PATH").unwrap_or_default();
    let path = format!("{}:{original_path}", bin_dir.display());

    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .env("PATH", path)
        .args(["tidy", "--no-learn"]);
    let out = command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin");

    assert!(
        !out.status.success(),
        "expected tidy to fail when post-ACP quality gates fail: {out:?}"
    );
    let trace_log = std::fs::read_to_string(&trace).unwrap_or_default();
    assert!(!trace_log.is_empty(), "expected quality gates to run after ACP: {trace_log}");
    assert!(
        trace_log.contains("kiss"),
        "expected at least one post-ACP quality gate command to run: {trace_log}"
    );
}

#[test]
fn malvin_sync_is_not_kiss_gated_when_kiss_missing_from_path() {
    assert_malvin_subcommand_not_kiss_gated_without_auth(&["sync", "--no-learn"]);
}

#[test]
fn malvin_kpop_is_not_kiss_gated_when_kiss_missing_from_path() {
    assert_malvin_subcommand_not_kiss_gated_without_auth(&["kpop", "x"]);
}
