//! Lazy `.malvin/checks` discovery via gate-loop commands (mock ACP).

#[cfg(unix)]
mod common;

#[cfg(unix)]
use std::fs;

#[cfg(unix)]
use common::{
    acp_mock_checks_discovery_and_code_js, acp_mock_checks_discovery_no_write_js,
    bin_path_with_fake_kiss, count_malvin_run_dirs, fast_test_home_workspace, seed_malvin_checks,
    spawn_malvin_code_discovery, CodeDiscoverySpawn,
};

#[cfg(unix)]
fn committed_repo_with_plan() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let (root, home, workspace) = fast_test_home_workspace();
    assert!(
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(&workspace)
            .status()
            .expect("git init")
            .success()
    );
    fs::write(workspace.join("README.md"), "hi\n").expect("write readme");
    fs::write(workspace.join("plan.md"), "build feature\n").expect("write plan");
    assert!(
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(&workspace)
            .status()
            .expect("git add")
            .success()
    );
    assert!(
        std::process::Command::new("git")
            .args(["commit", "-m", "initial"])
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@example.com")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@example.com")
            .current_dir(&workspace)
            .status()
            .expect("git commit")
            .success()
    );
    (root, home, workspace)
}

#[cfg(unix)]
#[test]
fn malvin_code_runs_checks_discovery_when_checks_missing() {
    let (root, home, workspace) = committed_repo_with_plan();
    let path = bin_path_with_fake_kiss(&root);
    let mock = acp_mock_checks_discovery_and_code_js();
    let out = spawn_malvin_code_discovery(&CodeDiscoverySpawn {
        project: &workspace,
        home: &home,
        mock_js: &mock,
        path_var: &path,
        request: "plan.md",
    });
    let combined = common::combined_cli_output(&out);
    assert!(
        out.status.success(),
        "malvin code with discovery should succeed: status={:?} combined={combined:?}",
        out.status
    );
    let checks = fs::read_to_string(workspace.join(".malvin/checks")).expect("checks");
    assert!(
        checks.lines().any(|l| l.trim() == "kiss check"),
        "discovery should write kiss check; got: {checks:?}"
    );
    assert!(
        count_malvin_run_dirs(&workspace, &home) >= 2,
        "expected discovery + code run dirs"
    );
}

#[cfg(unix)]
#[test]
fn malvin_code_fails_when_discovery_does_not_write_checks() {
    let (root, home, workspace) = committed_repo_with_plan();
    let path = bin_path_with_fake_kiss(&root);
    let mock = acp_mock_checks_discovery_no_write_js();
    let out = spawn_malvin_code_discovery(&CodeDiscoverySpawn {
        project: &workspace,
        home: &home,
        mock_js: &mock,
        path_var: &path,
        request: "plan.md",
    });
    let combined = common::combined_cli_output(&out);
    assert!(
        !out.status.success(),
        "malvin code should fail when discovery omits checks: {combined:?}"
    );
    assert!(
        combined.contains("checks discovery") || combined.contains(".malvin/checks still missing"),
        "expected discovery failure message: {combined:?}"
    );
}

#[cfg(unix)]
#[test]
fn malvin_code_skips_discovery_when_checks_preseeded() {
    let (root, home, workspace) = committed_repo_with_plan();
    seed_malvin_checks(&workspace, "kiss check\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = acp_mock_checks_discovery_no_write_js();
    let out = spawn_malvin_code_discovery(&CodeDiscoverySpawn {
        project: &workspace,
        home: &home,
        mock_js: &mock,
        path_var: &path,
        request: "plan.md",
    });
    let combined = common::combined_cli_output(&out);
    assert!(
        out.status.success(),
        "pre-seeded checks should skip discovery and run code: {combined:?}"
    );
    assert!(
        !combined.contains(".malvin/checks still missing"),
        "pre-seeded checks must skip discovery: {combined:?}"
    );
    let checks = fs::read_to_string(workspace.join(".malvin/checks")).expect("checks");
    assert!(checks.contains("kiss check"));
}
