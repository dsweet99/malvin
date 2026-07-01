use crate::repo_gates::discover_init_checks::*;
use crate::repo_gates::discover_init_checks_fixtures::{
    assert_deduped_precommit_checks, seed_enn_like_hybrid_fixture, seed_precommit_dedupe_fixture,
};
use crate::repo_gates::KISS_CHECK_COMMAND;
use std::fs;

/// Regression for enn: Python+Rust hybrid with `rust/Cargo.toml` (no root manifest),
/// Makefile `lint` runs clippy, pre-commit has ruff but no clippy hook.
#[test]
fn discover_init_check_commands_includes_clippy_from_makefile_when_precommit_omits_it() {
    let tmp = tempfile::tempdir().unwrap();
    seed_enn_like_hybrid_fixture(tmp.path());

    let lines = discover_init_check_commands(tmp.path());
    assert!(lines.first().is_some_and(|l| l == KISS_CHECK_COMMAND));
    assert!(
        lines.iter().any(|l| l.contains("cargo clippy")),
        "enn regression: expected makefile lint clippy when pre-commit omits it; got: {lines:?}"
    );
}

/// Regression for enn: finalize merges makefile clippy into `.malvin/checks` without
/// subprocess-spawning full `malvin init`.
#[test]
fn finalize_init_checks_from_repo_includes_clippy_for_enn_fixture() {
    let tmp = tempfile::tempdir().unwrap();
    seed_enn_like_hybrid_fixture(tmp.path());

    finalize_init_checks_from_repo(tmp.path()).unwrap();

    let checks = fs::read_to_string(crate::malvin_checks_path(tmp.path())).unwrap();
    assert!(
        checks.lines().any(|l| l.contains("cargo clippy")),
        "enn regression: expected clippy in .malvin/checks after finalize; got: {checks:?}"
    );
}

/// Regression: duplicate pre-commit ruff hooks and Makefile compileall collapse to one
/// ruff line plus pre-commit compileall (makefile must not override pre-commit).
#[test]
fn finalize_init_checks_from_repo_dedupes_precommit_fixture() {
    let tmp = tempfile::tempdir().unwrap();
    seed_precommit_dedupe_fixture(tmp.path());

    finalize_init_checks_from_repo(tmp.path()).unwrap();

    let checks = fs::read_to_string(crate::malvin_checks_path(tmp.path())).unwrap();
    assert_deduped_precommit_checks(&checks);
}
