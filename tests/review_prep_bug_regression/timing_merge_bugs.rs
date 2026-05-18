use super::helpers::{assert_tracked_in_git, manifest_root};
use std::process::Command;

#[test]
fn timing_merge_rs_must_be_tracked_when_wired_in_cli() {
    let cli_mod = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/cli/mod.rs"));
    if !cli_mod.contains("mod timing_merge") {
        return;
    }
    assert_tracked_in_git("src/cli/timing_merge.rs");
}

#[test]
fn timing_merge_unit_tests_build_in_binary_crate() {
    let out = Command::new("cargo")
        .args(["test", "-p", "malvin", "--bin", "malvin", "timing_merge", "--no-run"])
        .current_dir(manifest_root())
        .output()
        .expect("cargo test --bin malvin timing_merge --no-run");
    assert!(
        out.status.success(),
        "bug: cli timing_merge tests must compile; stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
