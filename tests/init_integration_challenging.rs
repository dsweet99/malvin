//! Challenging integration tests for `malvin init` discovery decision matrix.

mod common;

use std::fs;

use common::{
    assert_deduped_precommit_checks, gate_exp_logs_with_kpop_solved, git_init, malvin_init_output,
    only_run_dir, seed_enn_like_hybrid_fixture, seed_precommit_dedupe_fixture,
};

fn combined_output(out: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn malvin_init_empty_repo_skips_discovery_and_uses_builtin_checks() {
    let project = tempfile::tempdir().unwrap();
    assert!(!project.path().join(".git").exists());

    let (out, home) = malvin_init_output(project.path(), &["python"]);
    assert!(
        out.status.success(),
        "malvin init failed: {:?}",
        combined_output(&out)
    );

    let combined = combined_output(&out);
    assert!(
        combined.contains("empty repo; using builtin checks"),
        "expected empty-repo discovery skip message; got: {combined:?}"
    );

    let checks = fs::read_to_string(project.path().join(".malvin/checks")).expect("checks");
    assert!(
        checks.lines().any(|l| l.trim() == "kiss check"),
        "expected kiss builtin; got: {checks:?}"
    );
    assert!(
        checks.lines().any(|l| l.trim() == "ruff check ."),
        "python-only empty repo should seed ruff from init template; got: {checks:?}"
    );

    let run_dir = only_run_dir(project.path(), home.path());
    assert!(
        gate_exp_logs_with_kpop_solved(&run_dir).is_empty(),
        "empty repo must not run KPop discovery"
    );
}

#[test]
fn malvin_init_second_run_on_empty_repo_skips_discovery_without_agent() {
    let project = tempfile::tempdir().unwrap();
    let (out1, _home1) = malvin_init_output(project.path(), &["python"]);
    assert!(
        out1.status.success(),
        "first init failed: {:?}",
        combined_output(&out1)
    );
    let first_combined = combined_output(&out1);
    assert!(
        first_combined.contains("empty repo; using builtin checks"),
        "first init should take empty-repo path"
    );

    let (out2, _home2) = malvin_init_output(project.path(), &["python"]);
    assert!(
        out2.status.success(),
        "second init failed: {:?}",
        combined_output(&out2)
    );
    let combined = combined_output(&out2);
    assert!(
        combined.contains("checks already present; discovery skipped"),
        "second init should skip discovery because checks exist; got: {combined:?}"
    );
    assert!(
        !combined.contains("empty repo; using builtin checks"),
        "second init must not reuse empty-repo skip message; got: {combined:?}"
    );
}

#[test]
fn malvin_init_unborn_head_with_precommit_only_triggers_discovery() {
    let project = tempfile::tempdir().unwrap();
    git_init(project.path());
    fs::write(
        project.path().join(".pre-commit-config.yaml"),
        "repos:\n- repo: local\n  hooks:\n  - id: ruff\n    entry: ruff check .\n    language: system\n",
    )
    .expect("write pre-commit config");
    assert!(
        !std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(project.path())
            .output()
            .is_ok_and(|o| o.status.success()),
        "repo must have no commits before init"
    );

    let (out, home) = malvin_init_output(project.path(), &["python"]);
    assert!(
        out.status.success(),
        "malvin init failed: {:?}",
        combined_output(&out)
    );

    let combined = combined_output(&out);
    assert!(
        !combined.contains("empty repo; using builtin checks"),
        "pre-commit config alone should escape empty-repo fast path; got: {combined:?}"
    );
    assert!(
        !combined.contains("checks already present; discovery skipped"),
        "fresh tree should not skip discovery; got: {combined:?}"
    );

    let checks = fs::read_to_string(project.path().join(".malvin/checks")).expect("checks");
    assert!(
        checks.lines().any(|l| l.trim() == "kiss check"),
        "mock discovery should write kiss check; got: {checks:?}"
    );

    let run_dir = only_run_dir(project.path(), home.path());
    assert!(
        !gate_exp_logs_with_kpop_solved(&run_dir).is_empty(),
        "expected KPop discovery exp log with ## KPOP_SOLVED under {}",
        run_dir.display()
    );
}

#[test]
fn malvin_init_dedupes_precommit_hook_entries_into_checks() {
    let project = tempfile::tempdir().unwrap();
    seed_precommit_dedupe_fixture(project.path());

    let (out, _home) = malvin_init_output(project.path(), &["python"]);
    assert!(
        out.status.success(),
        "malvin init failed: {:?}",
        combined_output(&out)
    );

    let checks = fs::read_to_string(project.path().join(".malvin/checks")).expect("checks");
    assert_deduped_precommit_checks(&checks);
}

/// Regression for enn: Python+Rust hybrid (`rust/Cargo.toml`, no root manifest),
/// Makefile `lint` runs clippy, pre-commit has ruff but no clippy hook.
#[test]
fn malvin_init_python_rust_subdir_includes_clippy_from_makefile_lint() {
    let project = tempfile::tempdir().unwrap();
    seed_enn_like_hybrid_fixture(project.path());

    let (out, _home) = malvin_init_output(project.path(), &["python"]);
    assert!(
        out.status.success(),
        "malvin init failed: {:?}",
        combined_output(&out)
    );

    let checks = fs::read_to_string(project.path().join(".malvin/checks")).expect("checks");
    assert!(
        checks.lines().any(|l| l.contains("cargo clippy")),
        "enn regression: expected clippy in .malvin/checks after init; got: {checks:?}"
    );
}
