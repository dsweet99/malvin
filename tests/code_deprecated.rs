//! `malvin code` is deprecated at the CLI.

mod common;

use common::assert_code_deprecated;

#[test]
fn malvin_code_reports_deprecated() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"))
        .args(["code", "plan.md"])
        .output()
        .expect("spawn malvin code");
    assert_code_deprecated(&out);
}
