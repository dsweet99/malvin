//! Deprecated `malvin code --dry-run` flags remain mutually exclusive at parse time.

mod common;

#[cfg(unix)]
use common::{combined_cli_output, test_home_workspace};

#[cfg_attr(unix, test)]
fn dry_run_conflicts_with_trust_the_plan() {
    let (_root, home, workspace) = test_home_workspace();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"))
        .current_dir(&workspace)
        .env("HOME", &home)
        .args(["code", "--dry-run", "--trust-the-plan", "ship it"])
        .output()
        .expect("spawn malvin code");
    assert!(
        !out.status.success(),
        "expected clap conflict for --dry-run with --trust-the-plan: {out:?}"
    );
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("cannot be used with"),
        "expected clap mutual-exclusion error: {combined:?}"
    );
}
