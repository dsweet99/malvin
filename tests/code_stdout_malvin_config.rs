//! Deprecated `malvin code` home-config restore test.

#[cfg(unix)]
mod common;

#[cfg(unix)]
use common::{
    assert_code_deprecated, CodeSpawn, spawn_code, test_home_workspace,
    seed_git_kiss_cargo_gate_workspace, bin_path_with_fake_kiss, cached_mock_executable,
    acp_mock_kpop_tampers_home_malvin_config_writes_solved_js, workspace_kiss_check_only,
};

#[cfg(unix)]
#[test]
fn code_cli_is_deprecated() {
    let (root, home, workspace) = test_home_workspace();
    common::activate_test_home(&home);
    seed_git_kiss_cargo_gate_workspace(&workspace);
    workspace_kiss_check_only(&workspace);
    common::seed_malvin_config(&workspace, "mem_limit_gb = 7\n");
    let path = bin_path_with_fake_kiss(&root);
    let mock = cached_mock_executable(&acp_mock_kpop_tampers_home_malvin_config_writes_solved_js());
    let out = spawn_code(&CodeSpawn {
        workspace: &workspace,
        home: &home,
        mock: &mock,
        path_var: &path,
        extra_args: &["--max-loops", "1"],
        request: "ship it",
        gate_trace: None,
    });
    assert_code_deprecated(&out);
}
