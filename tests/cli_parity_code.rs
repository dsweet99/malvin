mod common;

#[cfg(unix)]
use common::{
    CodeSpawn, acp_mock_code_kpop_abort_result_js, bin_path_with_failing_gates,
    combined_cli_output, seed_git_kiss_cargo_gate_workspace, spawn_code, test_home_workspace,
    workspace_kiss_check_only, write_mock_executable,
};

#[cfg(unix)]
#[test]
fn code_stops_when_kpop_writes_abort_result() {
    let (root, home, workspace) = test_home_workspace();
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    let trace = root.path().join("kiss-trace.log");
    let path = bin_path_with_failing_gates(&root, &trace);
    let mock = root.path().join("mock-code-kpop-abort");
    write_mock_executable(&mock, &acp_mock_code_kpop_abort_result_js());
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
        "expected ABORT failure path: {out:?}"
    );
    let combined = combined_cli_output(&out);
    assert!(
        combined.contains("ABORT: code kpop stop"),
        "expected kpop ABORT to stop the workflow: {combined:?}"
    );
}
