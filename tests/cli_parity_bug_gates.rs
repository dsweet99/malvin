//! `malvin bug` skips workspace gates between KPOP (phase 1) and remediation.

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod unix_tests {
    use std::path::{Path, PathBuf};

    use super::common::{
        MALVIN_TEST_CMD_TIMEOUT, acp_mock_bug_kpop_solved_js, only_run_dir,
        seed_git_kiss_cargo_gate_workspace, test_home_workspace, write_failing_gate_tools,
        write_mock_executable,
    };

    struct BugGateSkipFixture {
        _root: tempfile::TempDir,
        workspace: PathBuf,
        home: PathBuf,
        mock: PathBuf,
        path: String,
        trace: PathBuf,
    }

    impl BugGateSkipFixture {
        fn new() -> Self {
            let (root, home, workspace) = test_home_workspace();
            seed_git_kiss_cargo_gate_workspace(&workspace);
            std::fs::write(workspace.join(".malvin_checks"), "kiss check\n").expect("malvin_checks");
            let bin_dir = root.path().join("bin");
            std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
            let trace = root.path().join("bug-post-kpop-gate-trace.log");
            write_failing_gate_tools(&bin_dir, &trace);
            let mock = root.path().join("mock-agent-acp-bug-post-kpop-gates");
            write_mock_executable(&mock, &acp_mock_bug_kpop_solved_js());
            let path = format!(
                "{}:{}",
                bin_dir.display(),
                std::env::var("PATH").unwrap_or_default()
            );
            Self {
                _root: root,
                workspace,
                home,
                mock,
                path,
                trace,
            }
        }

        fn spawn_hunt(&self) -> std::process::Output {
            let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"));
            cmd.current_dir(&self.workspace)
                .env("HOME", &self.home)
                .env("CURSOR_AGENT_API_KEY", "test-key")
                .env("MALVIN_AGENT_ACP_BIN", &self.mock)
                .env("PATH", &self.path)
                .args(["hunt", "--fix", "--no-learn", "--max-hypotheses", "1"]);
            super::common::command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT)
                .expect("spawn malvin hunt")
        }
    }

    fn assert_post_kpop_gate_skip(fx: &BugGateSkipFixture, combined: &str) {
        assert!(
            !combined.contains("Workspace checks did not pass"),
            "first-phase hunt must not run workspace gates after KPOP: {combined:?}"
        );
        assert!(
            combined.contains("Bug regression test"),
            "remediation must start after KPOP without post-KPOP gates: {combined:?}"
        );
        assert!(
            !fx.trace.exists(),
            "failing gate shims must not run when post-KPOP gates are skipped: {}",
            fx.trace.display()
        );
        assert_no_kiss_check_in_quality_gates_log(&fx.workspace);
    }

    fn assert_no_kiss_check_in_quality_gates_log(workspace: &Path) {
        let gates_log = only_run_dir(workspace).join("quality_gates.log");
        let gates_text = gates_log
            .exists()
            .then(|| std::fs::read_to_string(&gates_log).expect("quality_gates.log"));
        assert!(
            gates_text
                .as_deref()
                .is_none_or(|log| !log.contains("Running `kiss check`")),
            "post-KPOP hunt must not invoke kiss check via workspace gates: {gates_text:?}"
        );
    }

    #[test]
    fn post_kpop_skips_workspace_gates_before_remediation() {
        let fx = BugGateSkipFixture::new();
        let out = fx.spawn_hunt();
        let combined = super::common::combined_cli_output(&out);
        assert_post_kpop_gate_skip(&fx, &combined);
    }
}
