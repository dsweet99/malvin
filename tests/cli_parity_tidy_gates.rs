//! `malvin tidy` startup gate failure messaging.

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod unix_tests {
    use super::common::{
        TidySpawn, acp_mock_js, bin_path_with_failing_gates, chunk_line,
        seed_git_kiss_cargo_gate_workspace, spawn_tidy, test_home_workspace, write_mock_executable,
    };

    #[test]
    fn startup_gate_failure_surfaces_pre_check_guidance() {
        let (root, home, workspace) = test_home_workspace();
        seed_git_kiss_cargo_gate_workspace(&workspace);
        std::fs::write(workspace.join(".malvin_checks"), "kiss check\n").expect("malvin_checks");
        let trace = root.path().join("tidy-startup-gate-trace.log");
        let path = bin_path_with_failing_gates(&root, &trace);
        let mock = root.path().join("mock-agent-acp-tidy-startup-gates");
        write_mock_executable(&mock, &acp_mock_js("", &chunk_line("tidy agent turn")));
        let out = spawn_tidy(&TidySpawn {
            workspace: &workspace,
            home: &home,
            mock: &mock,
            path_var: &path,
            extra_args: &["--max-loops", "1"],
        });
        let combined = super::common::combined_cli_output(&out);
        assert!(
            combined.contains("Pre-checks failed"),
            "tidy startup gate failure should use pre-check guidance: {combined:?}"
        );
        assert!(
            combined.contains("retry `malvin tidy`"),
            "expected malvin tidy retry guidance: {combined:?}"
        );
        assert!(
            combined.contains("malvin tidy"),
            "expected tidy guidance in pre-check failure: {combined:?}"
        );
        assert!(
            combined.contains("--skip-pre-checks"),
            "expected skip-pre-checks guidance: {combined:?}"
        );
    }
}
