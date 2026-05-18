//! Bugs from `review_prep.md` § Bugs — collapsed `.kissignore` breaks `kiss check`.

use super::helpers::manifest_root;
use std::process::Command;

const HEAD_KISSIGNORE_EXCLUSIONS: &[&str] = &[
    "src/acp/reader_tests.rs",
    "src/acp/session_tests.rs",
    "src/acp/transport_tests.rs",
    "src/main.rs",
    "src/cli/repo_checks/",
];

fn kissignore_lines() -> Vec<String> {
    std::fs::read_to_string(manifest_root().join(".kissignore"))
        .expect("read .kissignore")
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

#[test]
fn kissignore_must_not_be_collapsed_to_target_only() {
    let lines = kissignore_lines();
    assert!(
        lines.len() > 1 || lines != ["target/"],
        "bug: .kissignore is collapsed to `target/` only; branch removes HEAD exclusions \
         (reader_tests.rs 1036 lines, main.rs depth, etc.)"
    );
}

#[test]
fn kissignore_must_list_oversized_acp_test_modules() {
    let lines = kissignore_lines();
    for entry in HEAD_KISSIGNORE_EXCLUSIONS {
        assert!(
            lines.iter().any(|l| l == entry),
            "bug: .kissignore missing `{entry}` (required for kiss check on this tree)"
        );
    }
}

#[test]
fn kiss_check_passes_with_committed_kissignore_policy() {
    let out = Command::new("kiss")
        .arg("check")
        .current_dir(manifest_root())
        .output()
        .expect("kiss check");
    assert!(
        out.status.success(),
        "bug: kiss check must pass with the repo .kissignore policy (not only after malvin \
         restores HEAD dotfiles); stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("NO VIOLATIONS"),
        "bug: expected NO VIOLATIONS from kiss check, got:\n{combined}"
    );
}
