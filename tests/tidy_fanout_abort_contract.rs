#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    TidySpawn, acp_mock_tidy_abort_after_first_coder_turn_js, bin_path_with_fake_kiss,
    seed_git_kiss_cargo_gate_workspace, seed_malvin_checks, spawn_tidy, test_home_workspace,
    write_mock_executable,
};

#[cfg(unix)]
fn prepare_tidy_gate_failure(workspace: &std::path::Path) {
    seed_git_kiss_cargo_gate_workspace(workspace);
    seed_malvin_checks(workspace, "false\n");
}

#[cfg_attr(unix, test)]
fn tidy_stops_when_first_coder_turn_writes_abort_before_review_fanout() {
    let (root, home, workspace) = test_home_workspace();
    prepare_tidy_gate_failure(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-tidy-implement-abort");
    write_mock_executable(&mock, &acp_mock_tidy_abort_after_first_coder_turn_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(
        !out.status.success(),
        "tidy must fail when the first coder turn writes ABORT: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("ABORT: tidy implement abort"),
        "expected implement-path ABORT in output: {combined:?}"
    );
    assert!(
        combined.contains("review_fanout = 0.0s"),
        "review fan-out must not run after implement ABORT (timing): {combined:?}"
    );
}
