//! Deprecated `malvin code` dry-run flag tests.

mod common;

use common::assert_code_deprecated;

#[test]
fn code_cli_is_deprecated_before_flag_parse() {
    let home = tempfile::tempdir().expect("home");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"))
        .env("HOME", home.path())
        .args(["code", "--dry-run", "--trust-the-plan", "ship it"])
        .output()
        .expect("spawn malvin code");
    assert_code_deprecated(&out);
}
