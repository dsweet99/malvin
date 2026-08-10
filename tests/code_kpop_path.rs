//! CLI entry does not require `kiss` on `PATH` for agent-backed subcommands.

#[cfg(unix)]
mod common;

use std::process::Command;

#[cfg(unix)]
use common::{MALVIN_TEST_CMD_TIMEOUT, command_output_with_timeout};

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

fn isolated_path_and_home() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let path_root = tempfile::tempdir().unwrap();
    let isolated_bin = path_root.path().join("bin");
    std::fs::create_dir_all(&isolated_bin).unwrap();
    let isolated_home = path_root.path().join("home");
    std::fs::create_dir_all(&isolated_home).unwrap();
    (path_root, isolated_bin, isolated_home)
}

fn assert_auth_failure_not_kiss_precheck(out: &std::process::Output) {
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

fn assert_malvin_subcommand_not_kiss_gated_without_auth(
    args: &[&str],
    work_dir: Option<&std::path::Path>,
) {
    let (_root, isolated_bin, isolated_home) = isolated_path_and_home();
    #[cfg(unix)]
    let out = run_malvin_path_timed(&isolated_bin, |c| {
        clear_agent_api_env(c);
        c.env("HOME", &isolated_home);
        if let Some(work_dir) = work_dir {
            c.current_dir(work_dir);
        }
        c.args(args);
    });
    #[cfg(not(unix))]
    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .env("PATH", &isolated_bin)
        .env("HOME", &isolated_home)
        .env_remove("CURSOR_AGENT_API_KEY")
        .env_remove("CURSOR_API_KEY")
        .env_remove("AGENT_API_KEY")
        .env_remove("MALVIN_AGENT_ACP_BIN")
        .args(args)
        .output()
        .expect("spawn malvin");
    assert_auth_failure_not_kiss_precheck(&out);
}

#[test]
fn malvin_tidy_is_not_kiss_gated_when_kiss_missing_from_path() {
    let work = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(work.path().join(".git")).unwrap();
    assert_malvin_subcommand_not_kiss_gated_without_auth(&["tidy"], Some(work.path()));
}

#[test]
fn write_skips_external_linter_preflight() {
    let work = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(work.path().join(".git")).unwrap();
    assert_malvin_subcommand_not_kiss_gated_without_auth(&["write", "topic"], Some(work.path()));
}

#[test]
fn malvin_do_is_not_kiss_gated_when_kiss_missing_from_path() {
    assert_malvin_subcommand_not_kiss_gated_without_auth(&["--do", "hello"], None);
}
