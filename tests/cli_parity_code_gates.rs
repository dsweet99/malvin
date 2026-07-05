//! `malvin code` gate-loop failure messaging when workspace checks fail.

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod unix_tests {
    use super::common::{
        CodeSpawn, acp_mock_code_kpop_steps_js, combined_cli_output, seed_malvin_checks,
        spawn_code, fast_test_home_workspace, cached_mock_executable,
        static_failing_gates_path_var,
    };

    #[test]
    fn gate_loop_failure_surfaces_guidance_message() {
        let (root, home, workspace) = fast_test_home_workspace();
        seed_malvin_checks(&workspace, "lint\n");
        let trace = root.path().join("gate-trace.log");
        let mock = cached_mock_executable( &acp_mock_code_kpop_steps_js());
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
        assert!(
            !out.status.success(),
            "malvin code should fail when gate loop exhausts with failing gates: {out:?}"
        );
        let combined = combined_cli_output(&out);
        assert!(
            combined.contains("ERR:"),
            "expected ERR-prefixed failure: {combined:?}"
        );
        assert!(
            combined.contains("Workspace checks did not pass")
                || combined.contains("quality gates"),
            "expected gate-loop failure message: {combined:?}"
        );
        assert!(
            combined.contains("retry `malvin code`") || combined.contains("malvin tidy"),
            "expected recovery guidance: {combined:?}"
        );
        assert!(
            combined.contains("lint") || trace.exists(),
            "expected gate failure detail or trace log: combined={combined:?} trace={}",
            trace.display()
        );
    }
}
