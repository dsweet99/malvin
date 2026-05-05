#[cfg(unix)]
mod common;

#[cfg(unix)]
mod unix_tests {
    use std::path::Path;
    use std::process::Command;

    use super::common::{
        MALVIN_TEST_CMD_TIMEOUT, acp_mock_code_streaming_update_js, command_output_with_timeout,
        seed_git_kiss_cargo_gate_workspace, test_home_workspace, write_failing_gate_tools,
        write_mock_executable,
    };

    struct PlanMockSpawn<'a> {
        workspace: &'a Path,
        home: &'a Path,
        mock_agent: &'a Path,
        path: &'a str,
    }

    fn spawn_malvin_plan_mock(sp: &PlanMockSpawn<'_>, plan_tail: &[&str]) -> std::process::Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_malvin"));
        cmd.current_dir(sp.workspace)
            .env("HOME", sp.home)
            .env("CURSOR_AGENT_API_KEY", "test-key")
            .env("MALVIN_AGENT_ACP_BIN", sp.mock_agent)
            .env("PATH", sp.path)
            .arg("plan");
        for a in plan_tail {
            cmd.arg(a);
        }
        command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT).expect("spawn malvin")
    }

    #[test]
    fn malvin_plan_runs_workspace_quality_gates_before_acp() {
        let (root, home, workspace) = test_home_workspace();
        seed_git_kiss_cargo_gate_workspace(&workspace);
        let bin_dir = root.path().join("bin");
        std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
        let trace = root.path().join("plan-gate-trace.log");
        write_failing_gate_tools(&bin_dir, &trace);
        let mock = root.path().join("mock-agent-acp-plan");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let original_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{original_path}", bin_dir.display());

        let sp = PlanMockSpawn {
            workspace: &workspace,
            home: &home,
            mock_agent: &mock,
            path: &path,
        };
        let out = spawn_malvin_plan_mock(&sp, &["minimal plan"]);

        assert!(
            !out.status.success(),
            "expected plan to fail when pre-session quality gates fail: {out:?}"
        );
        let trace_log = std::fs::read_to_string(&trace).unwrap_or_default();
        assert!(
            trace_log.contains("kiss"),
            "expected pre-ACP workspace gates to invoke kiss: {trace_log}"
        );
    }

    #[test]
    fn malvin_plan_skip_pre_checks_skips_workspace_gates() {
        let (root, home, workspace) = test_home_workspace();
        seed_git_kiss_cargo_gate_workspace(&workspace);
        let bin_dir = root.path().join("bin");
        std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
        let trace = root.path().join("plan-skip-gate-trace.log");
        write_failing_gate_tools(&bin_dir, &trace);
        let mock = root.path().join("mock-agent-acp-plan-skip");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let original_path = std::env::var("PATH").unwrap_or_default();
        let path = format!("{}:{original_path}", bin_dir.display());

        let sp = PlanMockSpawn {
            workspace: &workspace,
            home: &home,
            mock_agent: &mock,
            path: &path,
        };
        let out = spawn_malvin_plan_mock(&sp, &["--skip-pre-checks", "minimal plan"]);

        assert!(
            out.status.success(),
            "expected plan to pass when pre-checks skipped: {out:?}"
        );
        let trace_log = std::fs::read_to_string(&trace).unwrap_or_default();
        assert!(
            trace_log.is_empty(),
            "expected no gate tool invocations when --skip-pre-checks: {trace_log:?}"
        );
    }
}
