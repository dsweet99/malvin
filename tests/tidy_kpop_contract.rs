//! `malvin tidy` runs the kpop multiturn workflow with composed program/constraints prompts.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    TidySpawn, acp_mock_tidy_kpop_steps_js, bin_path_with_failing_gates, bin_path_with_fake_kiss,
    combined_cli_output, only_run_dir, seed_git_kiss_cargo_gate_workspace, spawn_tidy,
    test_home_workspace, workspace_kiss_check_only, write_mock_executable,
};

#[cfg(unix)]
#[test]
fn tidy_kpop_succeeds_when_steps_written_and_gates_pass() {
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
        "expected tidy success after kpop steps and passing gates: status={:?} combined={combined:?}",
        out.status,
    );
    assert!(
        combined.contains("KPOP_LOG:"),
        "tidy should emit kpop log line: {combined:?}"
    );
    let run_dir = only_run_dir(&workspace);
    let exp_dir = run_dir.join("_kpop");
    let exp_files: Vec<_> = std::fs::read_dir(&exp_dir)
        .expect("read kpop dir")
        .filter_map(Result::ok)
        .collect();
    assert!(!exp_files.is_empty(), "expected experiment log under _kpop");
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
