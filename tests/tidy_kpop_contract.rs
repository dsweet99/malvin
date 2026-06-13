//! `malvin tidy` runs the kpop multiturn workflow with composed program/constraints prompts.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    TidySpawn, acp_mock_kpop_tampers_gitignore_writes_solved_js, acp_mock_tidy_kpop_steps_js,
    bin_path_with_failing_gates, bin_path_with_fake_kiss, bin_path_with_kiss_fail_until_n_passes,
    combined_cli_output, seed_git_kiss_cargo_gate_workspace, spawn_tidy, test_home_workspace,
    workspace_kiss_check_only, write_mock_executable,
};

#[cfg(unix)]
#[test]
fn tidy_skips_kpop_when_gates_already_pass() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-tidy-kpop");
    write_mock_executable(&mock, &acp_mock_tidy_kpop_steps_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        out.status.success(),
        "expected tidy success when gates already pass: status={:?} combined={combined:?}",
        out.status,
    );
    assert!(combined.contains("DONE"), "expected fast-path DONE: {combined:?}");
    assert!(
        !combined.contains("KPOP_LOG:"),
        "tidy must skip kpop when gates pass before agent: {combined:?}"
    );
}

#[cfg(unix)]
#[test]
fn tidy_kpop_fails_when_post_session_gates_still_fail() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-trace.log");
    let path = bin_path_with_failing_gates(&root, &trace);
    let mock = root.path().join("mock-tidy-kpop-gates");
    write_mock_executable(&mock, &acp_mock_tidy_kpop_steps_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    assert!(
        !out.status.success(),
        "expected tidy to fail when post-kpop gates fail: {out:?}"
    );
    let trace_log = std::fs::read_to_string(&trace).unwrap_or_default();
    assert!(
        trace_log.contains("kiss"),
        "expected post-kpop quality gate run: {trace_log}"
    );
}

#[cfg(unix)]
#[test]
fn tidy_gate_loop_reconciles_gitignore_before_early_exit_gates() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    std::fs::write(workspace.join(".gitignore"), "gi\n").expect("gitignore");
    let trace = root.path().join("kiss-trace.log");
    let path = bin_path_with_kiss_fail_until_n_passes(&root, &trace, 1);
    let mock = root.path().join("mock-tidy-kpop-gitignore");
    write_mock_executable(&mock, &acp_mock_kpop_tampers_gitignore_writes_solved_js());
    let out = spawn_tidy(&TidySpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
    });
    let combined = combined_cli_output(&out);
    assert!(
        out.status.success(),
        "expected tidy early-exit success with reconciled gitignore: {combined:?}"
    );
    let gitignore = std::fs::read_to_string(workspace.join(".gitignore")).expect("read");
    assert!(
        gitignore.lines().any(|line| line.trim() == "ops/"),
        "expected canonical ops/ exclusion after restore+reconcile, got: {gitignore:?}"
    );
}
