#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    MALVIN_TEST_CMD_TIMEOUT, acp_mock_code_with_run_dir_js, code_review_fanout_writes_regression_test_and_non_lgtm,
    command_output_with_timeout, seed_git_kiss_cargo_gate_workspace, test_home_workspace,
    write_fake_kiss, write_mock_executable,
};
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
fn spawn_code_review_write_regression(
    workspace: &Path,
    home: &Path,
    mock: &Path,
    path_var: &str,
) -> std::process::Output {
    write_mock_executable(
        mock,
        &acp_mock_code_with_run_dir_js(&code_review_fanout_writes_regression_test_and_non_lgtm()),
    );
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(workspace)
        .env("HOME", home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", mock)
        .env("PATH", path_var)
        .args([
            "code",
            "--no-learn",
            "--max-loops",
            "1",
            "--skip-pre-checks",
            "--trust-the-plan",
            "ship it",
        ]);
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
}

#[cfg_attr(unix, test)]
fn code_review_write_fanout_writes_failing_regression_test_before_non_lgtm_review() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    let path = {
        let bin_dir = root.path().join("bin");
        std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
        write_fake_kiss(&bin_dir.join("kiss"));
        format!(
            "{}:{}",
            bin_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        )
    };
    let mock = root.path().join("mock-code-review-write-regression");
    let out = spawn_code_review_write_regression(&workspace, &home, &mock, &path);
    assert!(
        !out.status.success(),
        "expected code to fail when review is non-LGTM: {out:?}"
    );
    let regression = workspace.join("tests/review_write_fanout_regression.rs");
    let contents = std::fs::read_to_string(&regression).unwrap_or_else(|e| {
        panic!("review_write should create {regression:?}: {e}")
    });
    assert!(
        contents.contains("review_write_fanout_exposes_bug"),
        "expected failing regression test in {regression:?}: {contents}"
    );
    assert!(
        contents.contains("assert!(false)"),
        "regression test should fail when run: {contents}"
    );
}
