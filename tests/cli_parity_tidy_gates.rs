//! `malvin tidy` startup gate failure messaging.

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod unix_tests {
    use std::path::PathBuf;

    use super::common::{
        TidySpawn, acp_mock_tidy_kpop_steps_js, bin_path_with_failing_gates, combined_cli_output,
        seed_git_kiss_cargo_gate_workspace, seed_malvin_checks, spawn_tidy, test_home_workspace,
        write_mock_executable,
    };

    struct TidyStartupGateFixture {
        _root: tempfile::TempDir,
        workspace: PathBuf,
        home: PathBuf,
        mock: PathBuf,
        path: String,
    }

    impl TidyStartupGateFixture {
        fn new() -> Self {
            let (root, home, workspace) = test_home_workspace();
            seed_git_kiss_cargo_gate_workspace(&workspace);
            seed_malvin_checks(&workspace, "kiss check\n");
            let trace = root.path().join("tidy-startup-gate-trace.log");
            let path = bin_path_with_failing_gates(&root, &trace);
            let mock = root.path().join("mock-agent-acp-tidy-startup-gates");
            write_mock_executable(&mock, &acp_mock_tidy_kpop_steps_js());
            Self {
                _root: root,
                workspace,
                home,
                mock,
                path,
            }
        }

        fn spawn(&self) -> std::process::Output {
            spawn_tidy(&TidySpawn {
                workspace: &self.workspace,
                home: &self.home,
                mock: &self.mock,
                path_var: &self.path,
                extra_args: &["--max-loops", "1"],
            })
        }
    }

    #[test]
    fn startup_gate_failure_messaging_and_kpop_session() {
        let fx = TidyStartupGateFixture::new();
        let combined = combined_cli_output(&fx.spawn());
        assert!(
            !combined.contains("Pre-checks failed"),
            "tidy must not use code-style pre-check guidance: {combined:?}"
        );
        assert!(
            !combined.contains("implementation did not start"),
            "tidy startup gate failure must not claim implementation never started: {combined:?}"
        );
        assert!(
            combined.contains("kiss check"),
            "expected gate failure detail from repo checks: {combined:?}"
        );
        assert!(
            combined.contains("KPOP_LOG:"),
            "tidy should run kpop when startup gates fail: {combined:?}"
        );
    }
}
