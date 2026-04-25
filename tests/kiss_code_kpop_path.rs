//! `malvin code` fails fast when `kiss` is not on `PATH`.

use std::process::Command;

fn assert_malvin_subcommand_fails_without_kiss(args: &[&str]) {
    let path_root = tempfile::tempdir().unwrap();
    let isolated_bin = path_root.path().join("bin");
    std::fs::create_dir_all(&isolated_bin).unwrap();

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

#[test]
fn malvin_code_fails_fast_when_kiss_missing_from_path() {
    assert_malvin_subcommand_fails_without_kiss(&["code", "x"]);
}

#[test]
fn malvin_kpop_is_not_kiss_gated_when_kiss_missing_from_path() {
    let path_root = tempfile::tempdir().unwrap();
    let isolated_bin = path_root.path().join("bin");
    std::fs::create_dir_all(&isolated_bin).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .env("PATH", &isolated_bin)
        .env_remove("CURSOR_AGENT_API_KEY")
        .env_remove("CURSOR_API_KEY")
        .env_remove("AGENT_API_KEY")
        .env_remove("MALVIN_AGENT_ACP_BIN")
        .args(["kpop", "x"])
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
        "kpop should not fail on a kiss precheck; got: {msg:?}"
    );
}
