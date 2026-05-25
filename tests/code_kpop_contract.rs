//! `malvin code` runs the kpop multiturn gate-loop workflow with `code_constraints.md`.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    CodeSpawn, acp_mock_code_kpop_steps_js, bin_path_with_failing_gates, bin_path_with_fake_kiss,
    combined_cli_output, seed_git_kiss_cargo_gate_workspace, spawn_code,
    test_home_workspace, workspace_kiss_check_only, write_mock_executable,
};

#[cfg(unix)]
#[test]
fn code_runs_kpop_when_gates_already_pass() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let path = bin_path_with_fake_kiss(&root);
    let mock = root.path().join("mock-code-kpop");
    write_mock_executable(&mock, &acp_mock_code_kpop_steps_js());
    let out = spawn_code(&CodeSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
        request: "ship it",
    });
    let combined = combined_cli_output(&out);
    assert!(
        out.status.success(),
        "expected code success when gates already pass: status={:?} combined={combined:?}",
        out.status,
    );
    assert!(combined.contains("DONE"), "expected DONE after post-kpop gates: {combined:?}");
    assert!(
        combined.contains("KPOP_LOG:"),
        "code must run kpop even when gates pass before agent: {combined:?}"
    );
}

#[cfg(unix)]
#[test]
fn code_kpop_fails_when_post_session_gates_still_fail() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-trace.log");
    let path = bin_path_with_failing_gates(&root, &trace);
    let mock = root.path().join("mock-code-kpop-gates");
    write_mock_executable(&mock, &acp_mock_code_kpop_steps_js());
    let out = spawn_code(&CodeSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
        request: "ship it",
    });
    assert!(
        !out.status.success(),
        "expected code to fail when post-kpop gates fail: {out:?}"
    );
    let trace_log = std::fs::read_to_string(&trace).unwrap_or_default();
    assert!(
        trace_log.contains("kiss"),
        "expected post-kpop quality gate run: {trace_log}"
    );
}
