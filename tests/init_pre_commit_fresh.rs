//! Follow-on `malvin init` tests (fresh repo commit + existing repo behavior).

mod common;

use malvin::MALVIN_ADVICE_REL;
use malvin::MALVIN_CONFIG_REL;

use common::{
    InitOk, assert_git_branch_main, assert_git_head_commit_count, git_show_rev_path,
    malvin_init_output, malvin_init_output_in_place, tempdir_seeded_dirty_keep,
};

use malvin::repo_gates::{
    DEFAULT_PYTEST_CHECK, DEFAULT_RUST_CLIPPY, DEFAULT_RUST_NEXTEST, DEFAULT_RUST_NEXTEST_PARTITION_1,
    DEFAULT_RUST_NEXTEST_PARTITION_2, DEFAULT_RUST_TEST,
};

fn init_combined_output(out: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// Regression for plan.md: `malvin init rust` in an empty directory must seed clippy and
/// a cargo test runner in `.malvin/checks`, not only `kiss check`.
///
/// Uses in-place CWD (no `--path`) so the test catches entrypoint ordering that pre-seeds
/// kiss-only checks before `Cargo.toml` exists.
#[test]
fn malvin_init_rust_empty_directory_seeds_rust_quality_gates() {
    let project = tempfile::tempdir().unwrap();
    assert!(!project.path().join("Cargo.toml").exists());
    let (out, _home) = malvin_init_output_in_place(project.path(), &["rust"]);
    let combined = init_combined_output(&out);
    assert!(
        out.status.success(),
        "malvin init rust failed on empty directory: {combined:?}"
    );
    let checks = std::fs::read_to_string(project.path().join(".malvin/checks"))
        .expect("read .malvin/checks");
    let lines: Vec<&str> = checks.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.first(), Some(&"kiss check"), "kiss must be first; got: {lines:?}");
    assert!(
        lines.contains(&DEFAULT_RUST_CLIPPY),
        "expected clippy in checks; got: {checks:?}"
    );
    let has_test_runner = lines.iter().any(|l| {
        *l == DEFAULT_RUST_NEXTEST
            || *l == DEFAULT_RUST_TEST
            || *l == DEFAULT_RUST_NEXTEST_PARTITION_1
            || *l == DEFAULT_RUST_NEXTEST_PARTITION_2
    });
    assert!(
        has_test_runner,
        "expected cargo nextest run or cargo test in checks; got: {checks:?}"
    );
    assert!(
        project.path().join("Cargo.toml").is_file(),
        "init rust should create Cargo.toml before seeding checks"
    );
}

/// Regression for plan.md: `malvin init python` in an empty directory must seed ruff in
/// `.malvin/checks`, not only `kiss check`.
///
/// Uses in-place CWD (no `--path`) so the test catches entrypoint ordering that pre-seeds
/// kiss-only checks before init templates write `.pre-commit-config.yaml`.
#[test]
fn malvin_init_python_empty_directory_seeds_python_quality_gates() {
    let project = tempfile::tempdir().unwrap();
    assert!(!project.path().join(".pre-commit-config.yaml").exists());
    let (out, _home) = malvin_init_output_in_place(project.path(), &["python"]);
    let combined = init_combined_output(&out);
    assert!(
        out.status.success(),
        "malvin init python failed on empty directory: {combined:?}"
    );
    let checks = std::fs::read_to_string(project.path().join(".malvin/checks"))
        .expect("read .malvin/checks");
    let lines: Vec<&str> = checks.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.first(), Some(&"kiss check"), "kiss must be first; got: {lines:?}");
    assert!(
        lines.contains(&"ruff check ."),
        "expected ruff in checks; got: {checks:?}"
    );
    assert!(
        lines.contains(&DEFAULT_PYTEST_CHECK),
        "expected pytest in checks; got: {checks:?}"
    );
    assert!(
        project.path().join(".pre-commit-config.yaml").is_file(),
        "init python should create .pre-commit-config.yaml before seeding checks"
    );
}

/// Regression for bug.md: `malvin init python` in an empty directory must not fail on
/// `pre-commit install` before git exists.
#[test]
fn malvin_init_empty_directory_does_not_fail_pre_commit_install() {
    let project = tempfile::tempdir().unwrap();
    assert!(!project.path().join(".git").exists());
    let (out, _home) = malvin_init_output(project.path(), &["python"]);
    let combined = init_combined_output(&out);
    assert!(
        out.status.success(),
        "malvin init failed on empty directory: {combined:?}"
    );
    assert!(
        !combined.contains("`pre-commit install` failed"),
        "bug.md regression: pre-commit install must succeed after git init; got: {combined:?}"
    );
    assert!(project.path().join(".git").exists());
    assert_git_branch_main(project.path());
    assert!(
        project.path().join(".git/hooks/pre-commit").is_file(),
        "pre-commit hook should be installed"
    );
}

#[test]
fn malvin_init_creates_initial_commit_for_fresh_repo() {
    let w = InitOk::new(&["python"]);
    assert_git_head_commit_count(w.path(), "1");
    assert!(
        w.path().join(".malvin/checks").is_file(),
        "init must write .malvin/checks"
    );
    assert!(
        w.path().join(MALVIN_ADVICE_REL).is_file(),
        "init must write {MALVIN_ADVICE_REL}"
    );
    assert!(
        w.home_path().join(".malvin_home/config.toml").is_file(),
        "init must write ~/.malvin_home/config.toml"
    );
    assert!(
        !w.path().join(MALVIN_CONFIG_REL).exists(),
        "init must not write workspace-local config"
    );
}

#[test]
fn malvin_init_does_not_autocommit_preexisting_repo_changes() {
    let project = tempdir_seeded_dirty_keep();
    let (out, _home) = malvin_init_output(project.path(), &["python"]);
    assert!(out.status.success(), "malvin init failed: {out:?}");
    assert_git_head_commit_count(project.path(), "1");
    assert_eq!(
        git_show_rev_path(project.path(), "HEAD:keep.txt"),
        "before\n",
        "existing tracked content should not be silently rewritten into a new init commit"
    );
}
