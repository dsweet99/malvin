#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    TidySpawn, acp_mock_tidy_review_write_succeeds_on_third_inner_try_js,
    bin_path_with_kiss_fail_until_n_passes, only_run_dir, seed_git_kiss_cargo_gate_workspace,
    spawn_tidy, test_home_workspace, workspace_kiss_check_only, write_mock_executable,
};
#[cfg_attr(unix, test)]
fn tidy_inner_review_write_retries_allow_at_least_max_loops_per_outer_iteration() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-inner-retry-cap.log");
    let path = bin_path_with_kiss_fail_until_n_passes(&root, &trace, 1);
    let mock = root.path().join("mock-tidy-inner-retry-cap");
    write_mock_executable(&mock, &acp_mock_tidy_review_write_succeeds_on_third_inner_try_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "5"],
    });
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success(),
        "tidy must succeed when the third inner review_write writes the artifact and \
         inner retry cap is at least 3 (historically tied to --max-loops): {out:?}\n{combined}"
    );
    let run_dir = only_run_dir(&workspace);
    let tries_path = run_dir.join(".review_write_tries");
    assert_eq!(
        std::fs::read_to_string(&tries_path).expect("inner try counter"),
        "3",
        "mock succeeds on the third inner review_write within one outer iteration; run_dir={run_dir:?}"
    );
    assert!(
        !combined.contains("review: review_write did not write artifact review after retries"),
        "inner cap must not exhaust before the third review_write succeeds: {combined:?}"
    );
}
