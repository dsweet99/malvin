//! `malvin hunt` discover-only and fix-by-id (mock ACP).

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod unix_tests {
    use std::path::Path;

    use super::common::{
        MALVIN_TEST_CMD_TIMEOUT, acp_mock_bug_kpop_solved_js, seed_git_kiss_cargo_gate_workspace,
        test_home_workspace, write_mock_executable,
    };

    struct BugSpawn<'a> {
        workspace: &'a Path,
        home: &'a Path,
        mock: &'a Path,
        path: &'a str,
        extra_args: &'a [&'a str],
    }

    fn spawn_bug(sp: &BugSpawn<'_>) -> std::process::Output {
        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"));
        cmd.current_dir(sp.workspace)
            .env("HOME", sp.home)
            .env("CURSOR_AGENT_API_KEY", "test-key")
            .env("MALVIN_AGENT_ACP_BIN", sp.mock)
            .env("PATH", sp.path)
            .arg("hunt")
            .args(sp.extra_args);
        super::common::command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT)
            .expect("spawn malvin bug")
    }

    #[test]
    fn discover_only_emits_bug_id_and_done_without_plan_md() {
        let (root, home, workspace) = test_home_workspace();
        seed_git_kiss_cargo_gate_workspace(&workspace);
        let mock = root.path().join("mock-agent-acp-bug-discover");
        write_mock_executable(&mock, &acp_mock_bug_kpop_solved_js());
        let path = std::env::var("PATH").unwrap_or_default();
        let sp = BugSpawn {
            workspace: &workspace,
            home: &home,
            mock: &mock,
            path: &path,
            extra_args: &["--no-learn", "--max-hypotheses", "1"],
        };
        let out = spawn_bug(&sp);
        assert!(out.status.success(), "discover-only should succeed: {out:?}");
        let combined = super::common::combined_cli_output(&out);
        assert!(
            combined.contains("BUG_ID: M"),
            "expected BUG_ID line: {combined:?}"
        );
        assert!(
            combined.contains("BUG_LOG: M"),
            "expected BUG_LOG line: {combined:?}"
        );
        assert!(combined.contains("DONE"), "expected DONE: {combined:?}");
        let run_dir = std::fs::read_dir(workspace.join("_malvin"))
            .expect("malvin dir")
            .find_map(Result::ok)
            .expect("one run")
            .path();
        assert!(
            !run_dir.join("plan.md").exists(),
            "discover-only must not write remediation plan.md"
        );
    }

    fn bug_id_from_output(combined: &str) -> &str {
        combined
            .lines()
            .find_map(|line| {
                line.split("BUG_ID: ")
                    .nth(1)
                    .map(str::trim)
                    .filter(|s| s.starts_with('M') && s.len() == 6)
            })
            .expect("BUG_ID in discover output")
    }

    #[test]
    fn fix_by_id_resolves_id_from_prior_discover_run() {
        let (root, home, workspace) = test_home_workspace();
        seed_git_kiss_cargo_gate_workspace(&workspace);
        let mock = root.path().join("mock-agent-acp-bug-fix-id");
        write_mock_executable(&mock, &acp_mock_bug_kpop_solved_js());
        let path = std::env::var("PATH").unwrap_or_default();
        let sp = BugSpawn {
            workspace: &workspace,
            home: &home,
            mock: &mock,
            path: &path,
            extra_args: &["--no-learn", "--max-hypotheses", "1"],
        };
        let discover = spawn_bug(&sp);
        assert!(discover.status.success(), "seed discover: {discover:?}");
        let discover_out = super::common::combined_cli_output(&discover);
        let id = bug_id_from_output(&discover_out);
        let fix = spawn_bug(&BugSpawn {
            extra_args: &["--no-learn", "--skip-pre-checks", id],
            ..sp
        });
        let fix_out = super::common::combined_cli_output(&fix);
        assert!(
            !fix_out.contains("no BUG_ID"),
            "fix-by-id should resolve id: {fix_out:?}"
        );
        assert!(
            !fix_out.contains("ambiguous"),
            "fix-by-id should not see duplicate ids: {fix_out:?}"
        );
    }
}
