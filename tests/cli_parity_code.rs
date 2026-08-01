//! Deprecated `malvin code` gate-loop tests (CLI no longer runs the workflow).

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{assert_code_deprecated, spawn_code, CodeSpawn, fast_test_home_workspace, cached_mock_executable, seed_malvin_checks_legacy_fast, bin_path_with_fake_kiss, ABORT_CODE_TEST_ARGS, acp_mock_code_kpop_abort_result_js};

#[cfg(unix)]
#[test]
fn code_cli_is_deprecated() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks_legacy_fast(&workspace, "true\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_code_kpop_abort_result_js());
    let out = spawn_code(&CodeSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: ABORT_CODE_TEST_ARGS,
        request: "ship it",
        gate_trace: None,
    });
    assert_code_deprecated(&out);
}
