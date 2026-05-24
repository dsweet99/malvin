//! `malvin bug` post-KPOP workspace gate failure messaging.

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod unix_tests {
    use std::path::Path;

    use super::common::{
        MALVIN_TEST_CMD_TIMEOUT, acp_mock_bug_kpop_solved_js, seed_git_kiss_cargo_gate_workspace,
        test_home_workspace, write_failing_gate_tools, write_mock_executable,
    };

    struct BugSpawn<'a> {
        workspace: &'a Path,
        home: &'a Path,
        mock: &'a Path,
        path: &'a str,
    }

    fn spawn_bug(sp: &BugSpawn<'_>) -> std::process::Output {
        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"));
        cmd.current_dir(sp.workspace)
            .env("HOME", sp.home)
            .env("CURSOR_AGENT_API_KEY", "test-key")
            .env("MALVIN_AGENT_ACP_BIN", sp.mock)
            .env("PATH", sp.path)
            .args(["bughunt", "--no-learn", "--max-hypotheses", "1"]);
        super::common::command_output_with_timeout(&mut cmd, MALVIN_TEST_CMD_TIMEOUT)
            .expect("spawn malvin bughunt")
    }

    #[test]
    fn post_kpop_gate_failure_surfaces_workspace_guidance() {
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
        let sp = BugSpawn {
            workspace: &workspace,
            home: &home,
            mock: &mock,
            path: &path,
        };
        let out = spawn_bug(&sp);
        assert!(
            !out.status.success(),
            "malvin bughunt should fail when post-KPOP gates fail: {out:?}"
        );
        let combined = super::common::combined_cli_output(&out);
        assert!(
            combined.contains("ERR:"),
            "expected ERR-prefixed failure: {combined:?}"
        );
        assert!(
            combined.contains("Workspace checks did not pass"),
            "post-KPOP gate is not a startup pre-check: {combined:?}"
        );
        assert!(
            !combined.contains("implementation did not start"),
            "KPOP already ran; message must not claim implementation never started: {combined:?}"
        );
        assert!(
            combined.contains("malvin tidy"),
            "expected tidy guidance: {combined:?}"
        );
        assert!(
            combined.contains("retry `malvin bughunt`"),
            "expected malvin bughunt retry guidance: {combined:?}"
        );
        assert!(
            combined.contains("--skip-pre-checks"),
            "expected skip-pre-checks guidance: {combined:?}"
        );
    }
}
