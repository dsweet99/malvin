//! Deprecated `malvin code` gate-loop failure tests.

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod unix_tests {
    use super::common::{
        assert_code_deprecated, CodeSpawn, seed_malvin_checks, spawn_code,
        fast_test_home_workspace, cached_mock_executable, static_failing_gates_path_var,
        acp_mock_code_kpop_steps_js,
    };

    #[test]
    fn code_cli_is_deprecated() {
        let (root, home, workspace) = fast_test_home_workspace();
        seed_malvin_checks(&workspace, "lint\n");
        let trace = root.path().join("gate-trace.log");
        let mock = cached_mock_executable(&acp_mock_code_kpop_steps_js());
        let path = static_failing_gates_path_var();
        let out = spawn_code(&CodeSpawn {
            workspace: &workspace,
            home: &home,
            mock: &mock,
            path_var: &path,
            extra_args: &["--trust-the-plan", "--max-loops", "0"],
            request: "ship it",
            gate_trace: Some(&trace),
        });
        assert_code_deprecated(&out);
    }
}
