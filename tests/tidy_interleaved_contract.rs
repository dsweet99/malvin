#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    TidySpawn, acp_mock_tidy_fanout_lgtm_js, acp_mock_tidy_fanout_non_lgtm_js,
    bin_path_with_failing_gates, bin_path_with_fake_kiss, bin_path_with_kiss_fail_until_n_passes,
    only_run_dir,
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
fn tidy_interleaved_succeeds_when_reviewer_lgtm_and_gates_pass() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-tidy-lgtm-pass");
    write_mock_executable(&mock, &acp_mock_tidy_fanout_lgtm_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(
        out.status.success(),
        "expected tidy success when reviewer LGTM and kiss passes: status={:?} stdout={} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );
}

#[cfg_attr(unix, test)]
fn tidy_interleaved_writes_checks_marker_when_lgtm_and_in_loop_gates_fail() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-trace.log");
    let path = bin_path_with_failing_gates(&root, &trace);
    let mock = root.path().join("mock-tidy-lgtm-fail-gates");
    write_mock_executable(&mock, &acp_mock_tidy_fanout_lgtm_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(
        !out.status.success(),
        "expected tidy to fail when in-loop gates fail: {out:?}"
    );
    let run_dir = only_run_dir(&workspace);
    let review = std::fs::read_to_string(run_dir.join("review.md")).expect("read review");
    assert!(
        review.contains("Checks do not pass"),
        "expected artifact review after failed gates: {review:?}"
    );
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
        combined.contains("Concerns (attempt 1)")
            || combined.contains("tidy_concerns")
            || combined.contains(">tidy_concerns"),
        "malvin tidy must run concerns after non-LGTM review even with --max-loops 1: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn tidy_interleaved_one_iteration_exhausts_when_reviewer_never_lgtm() {
    let (root, home, workspace) = test_home_workspace();
    seed_tidy_workspace(&workspace);
    let trace = root.path().join("gate-trace.log");
    let path = bin_path_with_failing_gates(&root, &trace);
    let mock = root.path().join("mock-tidy-no-lgtm");
    write_mock_executable(&mock, &acp_mock_tidy_fanout_non_lgtm_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(!out.status.success(), "expected tidy failure: {out:?}");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("tidy did not converge within 1 iterations"),
        "expected convergence error: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn tidy_interleaved_second_iteration_runs_after_checks_marker_with_max_loops_two() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-trace-two.log");
    let path = bin_path_with_failing_gates(&root, &trace);
    let mock = root.path().join("mock-tidy-lgtm-two-iters");
    write_mock_executable(&mock, &acp_mock_tidy_fanout_lgtm_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "2"],
    });
    assert!(
        !out.status.success(),
        "expected tidy to fail after two iterations when gates never pass: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("tidy iteration 2/2"),
        "expected second coder iteration after LGTM plus failed in-loop gates: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn tidy_regression_last_budgeted_iteration_gate_fail_schedules_concerns_pass() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-trace-last-iter-gates.log");
    let path = bin_path_with_failing_gates(&root, &trace);
    let mock = root.path().join("mock-tidy-lgtm-gates-three-iters");
    write_mock_executable(&mock, &acp_mock_tidy_fanout_lgtm_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "3"],
    });
    assert!(!out.status.success(), "expected tidy failure when gates never pass: {out:?}");
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("tidy iteration 3/3"),
        "plan: after LGTM plus failed in-loop gates on the last budgeted iteration, run another tidy_concerns coder pass (expect a third iteration line before exit): {combined:?}"
    );
    assert!(
        combined.contains("tidy iteration 4/"),
        "expected an extra concerns-only iteration after the final budgeted review+gates failure: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn tidy_regression_bonus_concerns_reruns_gates_and_can_exit_zero() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-fail-once.log");
    let path = bin_path_with_kiss_fail_until_n_passes(&root, &trace, 2);
    let mock = root.path().join("mock-tidy-lgtm-gates-recover");
    write_mock_executable(&mock, &acp_mock_tidy_fanout_lgtm_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(
        out.status.success(),
        "after bonus concerns fixes gate failures, tidy must re-run gates and exit 0: {out:?}"
    );
}

#[cfg_attr(unix, test)]
fn tidy_regression_bonus_gate_recovery_artifact_review_is_lgtm_on_success() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-fail-bonus-review.log");
    let path = bin_path_with_kiss_fail_until_n_passes(&root, &trace, 2);
    let mock = root.path().join("mock-tidy-lgtm-gates-recover-review");
    write_mock_executable(&mock, &acp_mock_tidy_fanout_lgtm_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(
        out.status.success(),
        "expected tidy success after bonus concerns and gate recovery: {out:?}"
    );
    let run_dir = only_run_dir(&workspace);
    let artifact_path = run_dir.join("review.md");
    let artifact = std::fs::read_to_string(&artifact_path).expect("read artifact review");
    assert!(
        malvin::review_sync::is_lgtm_str(&artifact),
        "bonus gate recovery must re-run review before exit 0; artifact review was: {artifact:?}"
    );
    let workspace_review = workspace.join("review.md");
    if workspace_review.exists() {
        let ws = std::fs::read_to_string(&workspace_review).expect("read workspace review");
        assert!(
            !ws.contains("Checks do not pass"),
            "workspace review must not keep gate-failure marker after success: {ws:?}"
        );
    }
}
