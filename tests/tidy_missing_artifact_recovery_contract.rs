#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    TidySpawn, acp_mock_code_missing_artifact_recovers_on_outer_review_attempt_js,
    bin_path_with_fake_kiss, seed_git_kiss_cargo_gate_workspace, seed_malvin_checks, spawn_tidy,
    test_home_workspace,
    write_mock_executable,
};

#[cfg(unix)]
fn prepare_tidy_gate_failure(workspace: &std::path::Path) {
    seed_git_kiss_cargo_gate_workspace(workspace);
    seed_malvin_checks(workspace, "false\n");
}

#[cfg_attr(unix, test)]
fn tidy_missing_artifact_recovers_on_outer_iteration_with_new_review_prep() {
    let (root, home, workspace) = test_home_workspace();
    prepare_tidy_gate_failure(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-tidy-outer-refanout");
    write_mock_executable(
        &mock,
        &acp_mock_code_missing_artifact_recovers_on_outer_review_attempt_js(),
    );
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "2"],
    });
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("tidy iteration 2/2"),
        "tidy must reach second outer iteration when review_write can succeed: {combined:?}"
    );
    assert!(
        !combined.contains("review: review_write did not write artifact review after retries"),
        "inner exhaustion should not abort tidy when another outer iteration remains: {combined:?}"
    );
}
