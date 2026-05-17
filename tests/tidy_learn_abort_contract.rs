#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    TidySpawn, acp_mock_tidy_lgtm_abort_on_learn_js, bin_path_with_kiss_fail_until_n_passes,
    only_run_dir, seed_git_kiss_cargo_gate_workspace, spawn_tidy_with_learn, test_home_workspace,
    workspace_kiss_check_only, write_mock_executable,
};

#[cfg_attr(unix, test)]
fn tidy_aborts_before_summary_when_learn_writes_abort() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-learn-abort.log");
    let path = bin_path_with_kiss_fail_until_n_passes(&root, &trace, 1);
    let mock = root.path().join("mock-tidy-learn-abort");
    write_mock_executable(&mock, &acp_mock_tidy_lgtm_abort_on_learn_js());
    let out = spawn_tidy_with_learn(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(
        !out.status.success(),
        "tidy must fail when learn writes ABORT before summary: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("ABORT: tidy learn abort test"),
        "expected learn-path ABORT in output: {combined:?}"
    );
    assert!(
        !combined.contains(">summary"),
        "ABORT from learn must be honored before summary: {combined:?}"
    );
    let run_dir = only_run_dir(&workspace);
    let result = std::fs::read_to_string(run_dir.join("result.md")).expect("read result.md");
    assert!(
        result.contains("ABORT: tidy learn abort test"),
        "learn must write ABORT to result.md: {result:?}"
    );
}
