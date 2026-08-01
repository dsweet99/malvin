//! `malvin code` is deprecated at the CLI.

mod common;

use common::assert_code_deprecated;

#[test]
fn malvin_code_reports_deprecated() {
    let home = tempfile::tempdir().expect("home");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"))
        .env("HOME", home.path())
        .args(["code", "plan.md"])
        .output()
        .expect("spawn malvin code");
    assert_code_deprecated(&out);
}
