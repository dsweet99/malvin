#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    TidySpawn, acp_mock_tidy_fanout_lgtm_js, acp_mock_tidy_fanout_non_lgtm_js,
    acp_mock_tidy_fanout_skips_reviewer_outputs_js,
    acp_mock_tidy_review_write_succeeds_on_second_attempt_js, bin_path_with_fake_kiss,
    bin_path_with_kiss_fail_until_n_passes, only_run_dir, seed_git_kiss_cargo_gate_workspace,
    spawn_tidy,
    test_home_workspace, workspace_kiss_check_only, write_mock_executable,
};
#[cfg(unix)]
fn prepare_tidy_gate_failure(workspace: &std::path::Path) {
    seed_git_kiss_cargo_gate_workspace(workspace);
    std::fs::write(workspace.join(".malvin_checks"), "false\n").expect("checks");
}

#[cfg_attr(unix, test)]
fn tidy_interleaved_fails_when_fanout_reviewer_output_missing() {
    let (root, home, workspace) = test_home_workspace();
    prepare_tidy_gate_failure(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-tidy-missing-reviewer");
    write_mock_executable(&mock, &acp_mock_tidy_fanout_skips_reviewer_outputs_js());
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
        combined.contains("missing reviewer output"),
        "expected pre-aggregation fan-out preflight failure: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn tidy_interleaved_second_iteration_uses_tidy_concerns_after_non_lgtm_review() {
    let (root, home, workspace) = test_home_workspace();
    prepare_tidy_gate_failure(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-tidy-non-lgtm-two-iters");
    write_mock_executable(&mock, &acp_mock_tidy_fanout_non_lgtm_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "2"],
    });
    assert!(
        !out.status.success(),
        "expected tidy to fail after non-LGTM review: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("tidy iteration 2/2"),
        "expected second iteration after non-LGTM review_write: {combined:?}"
    );
}

#[cfg(unix)]
fn assert_run_timing_has_review_phases(run_dir: &std::path::Path) {
    let timing_text =
        std::fs::read_to_string(run_dir.join("run_timing.json")).expect("read run_timing.json");
    let timing: serde_json::Value =
        serde_json::from_str(&timing_text).expect("parse run_timing.json");
    let fanout_ms = timing["phases_ms"]["review_fanout"]
        .as_u64()
        .expect("review_fanout ms");
    let write_ms = timing["phases_ms"]["review_write"]
        .as_u64()
        .expect("review_write ms");
    assert!(fanout_ms > 0, "expected non-zero review_fanout timing: {timing_text}");
    assert!(write_ms > 0, "expected non-zero review_write timing: {timing_text}");
}

#[cfg_attr(unix, test)]
fn tidy_review_write_missing_artifact_retries_within_max_loops() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-tidy-review-write-retry");
    write_mock_executable(&mock, &acp_mock_tidy_review_write_succeeds_on_second_attempt_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "2"],
    });
    assert!(
        out.status.success(),
        "expected tidy to recover when review_write omits artifact on first try: {out:?}"
    );
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !combined.contains("review: review_write did not write artifact review after retries"),
        "tidy retry should recover from missing artifact on first review_write: {combined:?}"
    );
}

#[cfg_attr(unix, test)]
fn tidy_run_timing_records_review_fanout_and_write_phases() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-timing-trace.log");
    let path = bin_path_with_kiss_fail_until_n_passes(&root, &trace, 1);
    let mock = root.path().join("mock-tidy-timing-phases");
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
        "expected tidy success for timing test: {out:?}"
    );
    assert_run_timing_has_review_phases(&only_run_dir(&workspace));
}
