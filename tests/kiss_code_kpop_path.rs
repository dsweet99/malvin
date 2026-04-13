//! `malvin code` / `malvin kpop` fail fast when `kiss` is not on `PATH`.

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
fn malvin_kpop_fails_fast_when_kiss_missing_from_path() {
    assert_malvin_subcommand_fails_without_kiss(&["kpop", "x"]);
}
