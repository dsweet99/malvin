//! `malvin code` gate-loop failure messaging when workspace checks fail.

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod unix_tests {
    use super::common::{
        CodeSpawn, acp_mock_code_kpop_steps_js, combined_cli_output,
        seed_git_kiss_cargo_gate_workspace, spawn_code, test_home_workspace,
        write_failing_gate_tools, write_mock_executable,
    };

    #[test]
    fn gate_loop_failure_surfaces_guidance_message() {
        let (root, home, workspace) = test_home_workspace();
        seed_git_kiss_cargo_gate_workspace(&workspace);
        let bin_dir = root.path().join("bin");
        std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
        let trace = root.path().join("gate-trace.log");
        write_failing_gate_tools(&bin_dir, &trace);
        let mock = root.path().join("mock-agent-acp-code-gates");
        write_mock_executable(&mock, &acp_mock_code_kpop_steps_js());
        let path = format!(
            "{}:{}",
            bin_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        );
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
            combined.contains("kiss check") || trace.exists(),
            "expected gate failure detail or trace log: combined={combined:?} trace={}",
            trace.display()
        );
    }
}
