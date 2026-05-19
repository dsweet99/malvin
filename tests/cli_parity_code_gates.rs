//! `malvin code` workspace gate and pre-check failure messaging.

#[cfg(unix)]
mod common;

#[cfg(unix)]
mod unix_tests {
    use std::path::Path;

    use super::common::{
        acp_mock_code_review_lgtm_to_artifact_js, seed_git_kiss_cargo_gate_workspace,
        test_home_workspace, write_failing_gate_tools, write_mock_executable,
    };

    struct CodeSpawn<'a> {
        workspace: &'a Path,
        home: &'a Path,
        mock: &'a Path,
        path: &'a str,
        extra_args: &'a [&'a str],
    }

    fn spawn_code(sp: &CodeSpawn<'_>) -> std::process::Output {
        let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_malvin"));
        cmd.current_dir(sp.workspace)
            .env("HOME", sp.home)
            .env("CURSOR_AGENT_API_KEY", "test-key")
            .env("MALVIN_AGENT_ACP_BIN", sp.mock)
            .env("PATH", sp.path)
            .arg("code")
            .arg("--no-learn");
        for a in sp.extra_args {
            cmd.arg(a);
        }
        cmd.arg("ship it");
        super::common::command_output_with_timeout(&mut cmd, super::common::MALVIN_TEST_CMD_TIMEOUT)
            .expect("spawn malvin code")
    }

    #[test]
    fn pre_checks_failure_surfaces_guidance_message() {
        let (root, home, workspace) = test_home_workspace();
        seed_git_kiss_cargo_gate_workspace(&workspace);
        std::fs::write(workspace.join(".malvin_checks"), "kiss check\n").expect("malvin_checks");
        let bin_dir = root.path().join("bin");
        std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
        let trace = root.path().join("pre-check-trace.log");
        write_failing_gate_tools(&bin_dir, &trace);
        let mock = root.path().join("mock-agent-acp-code-pre");
        write_mock_executable(&mock, &acp_mock_code_review_lgtm_to_artifact_js());
        let path = format!(
            "{}:{}",
            bin_dir.display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let sp = CodeSpawn {
            workspace: &workspace,
            home: &home,
            mock: &mock,
            path: &path,
            extra_args: &[],
        };
        let out = spawn_code(&sp);
        assert!(
            !out.status.success(),
            "malvin code should fail when pre-check gates fail: {out:?}"
        );
        let combined = super::common::combined_cli_output(&out);
        assert!(
            combined.contains("ERR:"),
            "expected ERR-prefixed failure: {combined:?}"
        );
        assert!(
            combined.contains("Pre-checks failed"),
            "expected pre-check failure message: {combined:?}"
        );
        assert!(
            combined.contains("retry `malvin code`"),
            "expected explicit malvin code retry guidance: {combined:?}"
        );
        assert!(
            combined.contains("malvin tidy"),
            "expected tidy guidance in pre-check failure: {combined:?}"
        );
        assert!(
            combined.contains("--skip-pre-checks"),
            "expected skip-pre-checks guidance: {combined:?}"
        );
        assert!(
            combined.contains("kiss check") || trace.exists(),
            "expected gate failure detail or trace log: combined={combined:?} trace={}",
            trace.display()
        );
        assert!(
            !combined.contains("implemented"),
            "ACP implement should not run when pre-checks fail: {combined:?}"
        );
    }
}
