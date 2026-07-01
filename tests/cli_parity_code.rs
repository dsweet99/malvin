mod common;

#[cfg(unix)]
use common::{
    CodeSpawn, acp_mock_code_kpop_abort_result_js, bin_path_with_fake_kiss, combined_cli_output,
    fast_test_home_workspace, seed_malvin_checks, spawn_code, cached_mock_executable,
};

#[cfg(unix)]
#[test]
fn code_stops_when_kpop_writes_abort_result() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "kiss check\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_code_kpop_abort_result_js());
    let out = spawn_code(&CodeSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--trust-the-plan", "--max-loops", "0"],
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
