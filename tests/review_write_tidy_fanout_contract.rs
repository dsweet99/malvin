#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    MALVIN_TEST_CMD_TIMEOUT, acp_mock_code_with_run_dir_js, acp_mock_tidy_fanout_body,
    review_write_regression_test_body, seed_git_kiss_cargo_gate_workspace, test_home_workspace,
    write_artifact_non_lgtm, write_mock_executable,
};
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
fn acp_mock_tidy_review_write_regression_js() -> String {
    let write_tail = format!(
        "{}\n      {}\n",
        review_write_regression_test_body(),
        write_artifact_non_lgtm()
    );
    acp_mock_tidy_fanout_body(&write_tail)
}

#[cfg(unix)]
fn spawn_tidy_review_write_regression(
    workspace: &Path,
    home: &Path,
    mock: &Path,
    path_var: &str,
) -> std::process::Output {
    write_mock_executable(
        mock,
        &acp_mock_code_with_run_dir_js(&acp_mock_tidy_review_write_regression_js()),
    );
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(workspace)
        .env("HOME", home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", mock)
        .env("PATH", path_var)
        .args(["tidy", "--no-learn", "--max-loops", "1"]);
    common::command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
}

#[cfg_attr(unix, test)]
fn tidy_review_write_fanout_writes_failing_regression_test_before_non_lgtm_review() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    let path = common::bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-tidy-review-write-regression");
    let out = spawn_tidy_review_write_regression(&workspace, &home, &mock, &path);
    assert!(
        !out.status.success(),
        "expected tidy to fail when review is non-LGTM: {out:?}"
    );
    let regression = workspace.join("tests/review_write_fanout_regression.rs");
    let contents = std::fs::read_to_string(&regression)
        .unwrap_or_else(|e| panic!("review_write should create {regression:?}: {e}"));
    assert!(
        contents.contains("review_write_fanout_exposes_bug"),
        "expected failing regression test in {regression:?}: {contents}"
    );
    assert!(
        contents.contains("assert!(false)"),
        "regression test should fail when run: {contents}"
    );
}
