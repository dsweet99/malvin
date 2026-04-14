//! Product plan §1: `malvin init` fails fast with a clear message when `pre-commit` is not on `PATH`.

use std::process::Command;

#[test]
fn malvin_init_fails_fast_when_pre_commit_missing_from_path() {
    let path_root = tempfile::tempdir().unwrap();
    let isolated_bin = path_root.path().join("bin");
    std::fs::create_dir_all(&isolated_bin).unwrap();
    let project = tempfile::tempdir().unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .env("PATH", &isolated_bin)
        .args(["init", "python", "--path"])
        .arg(project.path())
        .output()
        .expect("spawn malvin init");

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
        msg.contains("pre-commit"),
        "expected explicit pre-commit hint; got: {msg:?}"
    );
}
