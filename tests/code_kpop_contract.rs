//! Deprecated `malvin code` kpop gate-loop contract tests.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    assert_code_deprecated, CodeSpawn, spawn_code, fast_test_home_workspace,
    seed_malvin_checks, bin_path_with_fake_kiss, cached_mock_executable,
    acp_mock_code_kpop_steps_js,
};

#[cfg(unix)]
#[test]
fn code_cli_is_deprecated() {
    let (root, home, workspace) = fast_test_home_workspace();
    seed_malvin_checks(&workspace, "true\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_code_kpop_steps_js());
    let out = spawn_code(&CodeSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--trust-the-plan", "--max-loops", "0"],
        request: "ship it",
        gate_trace: None,
    });
    assert_code_deprecated(&out);
}
