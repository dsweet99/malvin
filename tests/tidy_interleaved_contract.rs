#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    MALVIN_TEST_CMD_TIMEOUT, acp_mock_code_streaming_update_js, acp_mock_tidy_reviewer_lgtm_js,
    command_output_with_timeout, only_run_dir, seed_git_kiss_cargo_gate_workspace,
    test_home_workspace, write_fake_kiss, write_failing_gate_tools, write_mock_executable,
};
#[cfg(unix)]
use malvin::orchestrator::clear_review_file;
#[cfg(unix)]
use malvin::review_sync::{is_lgtm_str, sync_review_file_for_attempt};
#[cfg(unix)]
use std::path::{Path, PathBuf};
#[cfg(unix)]
use std::process::Command;

#[cfg(unix)]
struct TidySpawn<'a> {
    workspace: &'a Path,
    home: &'a Path,
    mock: &'a Path,
    path_var: &'a str,
    extra_args: &'a [&'a str],
}

#[cfg(unix)]
fn spawn_tidy(t: &TidySpawn<'_>) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
    cmd.current_dir(t.workspace)
        .env("HOME", t.home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", t.mock)
        .env("PATH", t.path_var);
    let mut args: Vec<&str> = vec!["tidy", "--no-learn"];
    args.extend_from_slice(t.extra_args);
    cmd.args(args);
    command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
}

#[cfg(unix)]
fn seed_tidy_workspace(workspace: &Path) {
    seed_git_kiss_cargo_gate_workspace(workspace);
    std::fs::write(workspace.join("script.py"), "print('broken')\n").expect("write python file");
}

#[cfg(unix)]
fn workspace_kiss_check_only(workspace: &Path) {
    std::fs::write(workspace.join(".malvin_checks"), "kiss check\n").expect("checks");
}

#[cfg(unix)]
fn bin_path_with_fake_kiss(root: &tempfile::TempDir) -> String {
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_fake_kiss(&bin_dir.join("kiss"));
    format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

#[cfg(unix)]
fn plan_item5_stale_lgtm_review_paths() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let t = tempfile::tempdir().expect("tempdir");
    let artifact = t.path().join("_malvin").join("run").join("review.md");
    let workspace = t.path().join("review.md");
    std::fs::create_dir_all(artifact.parent().expect("parent")).expect("mkdir");
    std::fs::write(&artifact, "LGTM\n").expect("artifact");
    std::fs::write(&workspace, "LGTM\n").expect("workspace");
    (t, artifact, workspace)
}

#[cfg(unix)]
fn bin_path_with_failing_gates(root: &tempfile::TempDir, trace: &Path) -> String {
    let bin_dir = root.path().join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_failing_gate_tools(&bin_dir, trace);
    format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

#[cfg_attr(unix, test)]
fn tidy_interleaved_succeeds_when_reviewer_lgtm_and_gates_pass() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-tidy-lgtm-pass");
    write_mock_executable(&mock, &acp_mock_tidy_reviewer_lgtm_js());
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
    write_mock_executable(&mock, &acp_mock_tidy_reviewer_lgtm_js());
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
fn tidy_interleaved_one_iteration_exhausts_when_reviewer_never_lgtm() {
    let (root, home, workspace) = test_home_workspace();
    seed_tidy_workspace(&workspace);
    let trace = root.path().join("gate-trace.log");
    let path = bin_path_with_failing_gates(&root, &trace);
    let mock = root.path().join("mock-tidy-no-lgtm");
    write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
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
    write_mock_executable(&mock, &acp_mock_tidy_reviewer_lgtm_js());
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
fn tidy_interleaved_plan_item5_stale_lgtm_cleared_before_reviewer_sync_contract() {
    let (_t, artifact, workspace) = plan_item5_stale_lgtm_review_paths();
    clear_review_file(&artifact).expect("clear artifact");
    clear_review_file(&workspace).expect("clear workspace");
    let synced = sync_review_file_for_attempt(&artifact, &workspace).expect("sync");
    assert!(
        !synced.as_deref().is_some_and(is_lgtm_str),
        "plan section 5: stale LGTM must not survive the same double-clear prelude as run_review_tidy_turn"
    );
}
