#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    TidySpawn, acp_mock_tidy_fanout_non_lgtm_js, acp_mock_tidy_fanout_non_lgtm_then_lgtm_js,
    bin_path_with_failing_gates, bin_path_with_kiss_fail_until_n_passes, only_run_dir,
    seed_git_kiss_cargo_gate_workspace, spawn_tidy, test_home_workspace, workspace_kiss_check_only,
    write_mock_executable,
};
#[cfg(unix)]
use std::path::Path;

#[cfg(unix)]
fn seed_tidy_workspace(workspace: &Path) {
    seed_git_kiss_cargo_gate_workspace(workspace);
    std::fs::write(workspace.join("script.py"), "print('broken')\n").expect("write python file");
}

#[cfg_attr(unix, test)]
fn tidy_max_loops_one_runs_concerns_after_non_lgtm_review() {
    let (root, home, workspace) = test_home_workspace();
    seed_tidy_workspace(&workspace);
    let trace = root.path().join("gate-trace-concerns.log");
    let path = bin_path_with_failing_gates(&root, &trace);
    let mock = root.path().join("mock-tidy-concerns-one-iter");
    write_mock_executable(&mock, &acp_mock_tidy_fanout_non_lgtm_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !out.status.success(),
        "gates never pass in this mock; tidy must not claim success: {out:?}"
    );
    assert!(
        combined.contains("Concerns (attempt 1)")
            || combined.contains("tidy_concerns")
            || combined.contains(">tidy_concerns"),
        "malvin tidy must run concerns after non-LGTM review even with --max-loops 1: {combined:?}"
    );
    assert!(
        combined.contains("tidy did not converge within 1 iterations"),
        "expected convergence failure after concerns when gates stay red: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn tidy_max_loops_one_non_lgtm_concerns_recovery_can_exit_zero() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-fail-non-lgtm-recover.log");
    let path = bin_path_with_kiss_fail_until_n_passes(&root, &trace, 1);
    let mock = root.path().join("mock-tidy-non-lgtm-recover");
    write_mock_executable(&mock, &acp_mock_tidy_fanout_non_lgtm_then_lgtm_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(
        out.status.success(),
        "after concerns on max-loops-1 non-LGTM, tidy must re-run gates and review and exit 0: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("tidy recovery (review attempt 2, max-loops 1)"),
        "max-loops-1 post-concerns recovery must not print a budgeted iteration overflow: {combined:?}"
    );
    let run_dir = only_run_dir(&workspace);
    let artifact = std::fs::read_to_string(run_dir.join("review.md")).expect("read artifact review");
    assert!(
        malvin::review_sync::is_lgtm_str(&artifact),
        "post-recovery review must be LGTM in run_dir artifact: {artifact:?}"
    );
    assert!(
        run_dir.join("reviewers_spawn_attempt_1.log").is_file(),
        "first review fan-out must leave attempt-1 spawn log; run_dir={run_dir:?}"
    );
    assert!(
        run_dir.join("reviewers_spawn_attempt_2.log").is_file(),
        "post-concerns recovery must use a distinct log attempt (like bonus recovery) \
         so it does not overwrite attempt-1 transcripts; run_dir={run_dir:?}"
    );
}
