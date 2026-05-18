use super::helpers::{
    assert_tracked_in_git, collect_untracked_path_wired_modules, git_status_short_lines,
    manifest_root,
};
use std::process::Command;

#[test]
fn cargo_discovers_review_prep_bug_regression_integration_test() {
    let out = Command::new("cargo")
        .args(["test", "--test", "review_prep_bug_regression", "--no-run"])
        .current_dir(manifest_root())
        .output()
        .expect("cargo test --no-run");
    assert!(
        out.status.success(),
        "bug: Cargo must register `tests/review_prep_bug_regression/main.rs` (or \
         `tests/review_prep_bug_regression.rs`) as integration test \
         `review_prep_bug_regression`; `mod.rs` alone is invisible: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn review_prep_bug_regression_integration_entry_is_main_rs_not_mod_rs() {
    let dir = manifest_root().join("tests/review_prep_bug_regression");
    let main_rs = dir.join("main.rs");
    let mod_rs = dir.join("mod.rs");
    let flat_rs = manifest_root().join("tests/review_prep_bug_regression.rs");
    assert!(
        main_rs.is_file() || flat_rs.is_file(),
        "bug: need tests/review_prep_bug_regression/main.rs or tests/review_prep_bug_regression.rs \
         for Cargo to run regression tests"
    );
    assert!(
        !mod_rs.is_file(),
        "bug: tests/review_prep_bug_regression/mod.rs is not a Cargo integration-test root; \
         use main.rs instead"
    );
}

#[test]
fn review_prep_bug_regression_exists_in_head_commit() {
    let out = Command::new("git")
        .args([
            "cat-file",
            "-e",
            "HEAD:tests/review_prep_bug_regression/main.rs",
        ])
        .current_dir(manifest_root())
        .output()
        .expect("git cat-file");
    assert!(
        out.status.success(),
        "bug: tests/review_prep_bug_regression/ must be committed on HEAD (clone/CI); \
         only staged or untracked copies are invisible to other checkouts"
    );
}

#[test]
fn review_prep_bug_regression_worktree_matches_git_index() {
    let am: Vec<String> = git_status_short_lines()
        .into_iter()
        .filter(|line| {
            line.starts_with("AM ") && line.contains("tests/review_prep_bug_regression/")
        })
        .collect();
    assert!(
        am.is_empty(),
        "bug: staged index for review_prep_bug_regression differs from worktree (AM); \
         commit would omit latest regression guards:\n{}",
        am.join("\n")
    );
}

#[test]
fn review_prep_bug_regression_sources_tracked_in_git() {
    for rel in [
        "tests/review_prep_bug_regression/main.rs",
        "tests/review_prep_bug_regression/helpers.rs",
        "tests/review_prep_bug_regression/git_and_wiring.rs",
        "tests/review_prep_bug_regression/stringify_and_orchestrator.rs",
    ] {
        assert_tracked_in_git(rel);
    }
}

#[test]
fn acp_reader_tests_must_be_wired_in_acp_module() {
    let acp_mod = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/acp/mod.rs"));
    assert!(
        acp_mod.contains("acp_reader_tests/mod.rs"),
        "bug: src/acp_reader_tests/ must be declared from acp/mod.rs so reader tests run"
    );
}

#[test]
fn kiss_stringify_cov_must_not_reference_removed_tidy_flow_helpers_module() {
    let src = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/cli/kiss_stringify_cov.rs"
    ));
    assert!(
        !src.contains("tidy_flow::helpers::"),
        "bug: kiss_stringify_cov.rs still stringify!s `tidy_flow::helpers::…` after helpers \
         were moved to internal `tidy_flow_helpers` (see src/cli/tidy_flow.rs); paths are stale"
    );
}

#[test]
fn cli_repo_checks_mod_rs_must_be_tracked_in_git() {
    assert_tracked_in_git("src/cli_repo_checks/mod.rs");
}

#[test]
fn all_path_wired_crate_modules_must_be_tracked_in_git() {
    let untracked = collect_untracked_path_wired_modules();
    assert!(
        untracked.is_empty(),
        "bug: #[path = …] modules wired in the crate must be in git (clone/CI break otherwise):\n{}",
        untracked.join("\n")
    );
}

#[test]
fn git_index_must_not_stage_rs_files_deleted_in_worktree() {
    let ad_deleted: Vec<String> = git_status_short_lines()
        .into_iter()
        .filter(|line| {
            line.starts_with("AD ")
                && (line.contains("src/") || line.contains("tests/"))
                && line.contains(".rs")
        })
        .collect();
    assert!(
        ad_deleted.is_empty(),
        "bug: git index stages .rs files that the working tree deleted (inc layout); \
         committing the index resurrects wrong sources and can break the build:\n{}",
        ad_deleted.join("\n")
    );
}
