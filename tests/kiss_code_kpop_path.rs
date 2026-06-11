//! `malvin code` fails fast when `kiss` is not on `PATH`.

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

fn assert_malvin_subcommand_not_kiss_gated_without_auth(
    args: &[&str],
    work_dir: Option<&std::path::Path>,
) {
    let path_root = tempfile::tempdir().unwrap();
    let isolated_bin = path_root.path().join("bin");
    std::fs::create_dir_all(&isolated_bin).unwrap();
    #[cfg(unix)]
    let out = run_malvin_path_timed(&isolated_bin, |c| {
        clear_agent_api_env(c);
        if let Some(work_dir) = work_dir {
            c.current_dir(work_dir);
        }
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
fn delight_does_not_require_kiss_on_path() {
    let work = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(work.path().join(".git")).unwrap();
    assert_malvin_subcommand_not_kiss_gated_without_auth(&["delight"], Some(work.path()));
}

#[test]
fn explain_does_not_require_kiss_on_path() {
    let work = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(work.path().join(".git")).unwrap();
    assert_malvin_subcommand_not_kiss_gated_without_auth(&["explain", "topic"], Some(work.path()));
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
fn malvin_do_is_not_kiss_gated_when_kiss_missing_from_path() {
    assert_malvin_subcommand_not_kiss_gated_without_auth(&["do", "hello"], None);
}

#[test]
fn malvin_kpop_is_not_kiss_gated_when_kiss_missing_from_path() {
    assert_malvin_subcommand_not_kiss_gated_without_auth(&["kpop", "x"], None);
}
