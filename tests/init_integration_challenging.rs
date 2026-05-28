//! Challenging integration tests for `malvin init` discovery/summary decision matrix.

mod common;

use std::fs;

use common::{
    gate_exp_logs_with_kpop_solved, git_init, malvin_init_output, only_run_dir,
};

fn combined_output(out: &std::process::Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

#[test]
fn malvin_init_empty_repo_skips_discovery_and_summary_uses_builtin_checks() {
    let project = tempfile::tempdir().unwrap();
    assert!(!project.path().join(".git").exists());

    let out = malvin_init_output(project.path(), &["python"]);
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
    assert!(
        !combined.contains("init summary ok"),
        "empty-repo fast path must not run summary agent; got: {combined:?}"
    );

    let checks = fs::read_to_string(project.path().join(".malvin/checks")).expect("checks");
    assert!(
        checks.lines().any(|l| l.trim() == "kiss check"),
        "expected kiss builtin; got: {checks:?}"
    );
    assert!(
        !checks.contains("ruff"),
        "python-only empty repo should not seed ruff; got: {checks:?}"
    );

    let run_dir = only_run_dir(project.path());
    assert!(
        gate_exp_logs_with_kpop_solved(&run_dir).is_empty(),
        "empty repo must not run KPop discovery"
    );
}

#[test]
fn malvin_init_second_run_on_empty_repo_runs_summary_without_discovery() {
    let project = tempfile::tempdir().unwrap();
    let out1 = malvin_init_output(project.path(), &["python"]);
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
    assert!(
        !first_combined.contains("init summary ok"),
        "first init on empty tree must not run summary"
    );

    let out2 = malvin_init_output(project.path(), &["python"]);
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
        combined.contains("init summary ok"),
        "second init on still-empty tree should run summary when skip reason is not empty-repo; got: {combined:?}"
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

    let out = malvin_init_output(project.path(), &["python"]);
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

    let run_dir = only_run_dir(project.path());
    assert!(
        !gate_exp_logs_with_kpop_solved(&run_dir).is_empty(),
        "expected KPop discovery exp log with ## KPOP_SOLVED under {}",
        run_dir.display()
    );
}
