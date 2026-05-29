use crate::repo_gates::discover_init_checks::*;
use crate::repo_gates::discover_init_checks_fixtures::seed_enn_like_hybrid_fixture;
use crate::repo_gates::KISS_CHECK_COMMAND;

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
