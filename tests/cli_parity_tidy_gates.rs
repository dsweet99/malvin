//! `malvin tidy` gate failure messaging under the router-backed path.

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod unix_tests {
    use std::path::PathBuf;

    use super::common::{
        TidySpawn, acp_mock_router_no_work_js, bin_path_with_failing_gates, combined_cli_output,
        seed_malvin_checks, spawn_tidy, fast_test_home_workspace, cached_mock_executable,
    };

    struct TidyGateFixture {
        _root: tempfile::TempDir,
        workspace: PathBuf,
        home: PathBuf,
        mock: PathBuf,
        path: String,
        trace: PathBuf,
    }

    impl TidyGateFixture {
        fn new() -> Self {
            let (root, home, workspace) = fast_test_home_workspace();
            seed_malvin_checks(&workspace, "lint\n");
            let trace = root.path().join("tidy-gate-trace.log");
            let path = bin_path_with_failing_gates(&root, &trace);
            let mock = cached_mock_executable(&acp_mock_router_no_work_js());
            Self {
                _root: root,
                workspace,
                home,
                mock,
                path,
                trace,
            }
        }

        fn spawn(&self) -> std::process::Output {
            spawn_tidy(&TidySpawn {
                workspace: &self.workspace,
                home: &self.home,
                mock: &self.mock,
                path_var: &self.path,
                extra_args: &["--max-loops", "0"],
                gate_trace: Some(&self.trace),
            })
        }
    }

    #[test]
    fn gate_failure_messaging_and_router_session() {
        let fx = TidyGateFixture::new();
        let combined = combined_cli_output(&fx.spawn());
        assert!(
            !combined.contains("Pre-checks failed"),
            "tidy must not use code-style pre-check guidance: {combined:?}"
        );
        assert!(
            !combined.contains("implementation did not start"),
            "tidy gate failure must not claim implementation never started: {combined:?}"
        );
        assert!(
            combined.contains("lint") || combined.contains("quality gates"),
            "expected gate failure detail from repo checks: {combined:?}"
        );
        assert!(
            combined.contains("Get the gates to pass.")
                || combined.contains("router_a")
                || combined.contains("__MALVIN_DONE__")
                || combined.contains("router_header"),
            "tidy should run the default router when gates fail: {combined:?}"
        );
    }
}
