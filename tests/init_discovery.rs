//! `malvin init` checks-discovery on existing repos (committed tree, mock ACP).

mod common;

use std::fs;
use std::path::Path;

use common::{
    gate_exp_logs_in_run, git_commit_all, git_init, malvin_init_output, only_run_dir,
};

fn gate_exp_logs_with_kpop_solved(run_dir: &Path) -> Vec<std::path::PathBuf> {
    gate_exp_logs_in_run(run_dir)
        .into_iter()
        .filter(|p| fs::read_to_string(p).is_ok_and(|text| text.contains("## KPOP_SOLVED")))
        .collect()
}

fn committed_repo_with_readme() -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    git_init(project.path());
    fs::write(project.path().join("README.md"), "hi\n").expect("write readme");
    git_commit_all(project.path(), "initial");
    project
}

fn assert_init_succeeded(out: &std::process::Output) {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "malvin init failed (exit {:?}): stdout={stdout} stderr={stderr}",
        out.status.code()
    );
}

#[test]
fn malvin_init_runs_discovery_on_committed_existing_repo() {
    let project = committed_repo_with_readme();
    let out = malvin_init_output(project.path(), &["python"]);
    assert_init_succeeded(&out);

    let checks = fs::read_to_string(project.path().join(".malvin/checks")).expect("checks");
    assert!(
        checks.lines().any(|l| l.trim() == "kiss check"),
        "mock discovery should write kiss check; got: {checks:?}"
    );

    let run_dir = only_run_dir(project.path());
    assert!(
        !gate_exp_logs_with_kpop_solved(&run_dir).is_empty(),
        "expected at least one gate exp log with ## KPOP_SOLVED under {}",
        run_dir.display()
    );

    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains("empty repo; using builtin checks"),
        "committed repo should not take empty-repo discovery skip"
    );
}
