mod common;

#[cfg(all(unix, target_os = "linux"))]
mod linux_pty {
    use crate::common::{
        acp_mock_code_streaming_update_js, acp_mock_kpop_tamper_then_restore_js,
        bin_path_with_fake_kiss, only_run_dir, run_do_under_script,
        run_kpop_multiturn_investigate, run_malvin_under_script_with_mock, spawn_kpop,
        test_home_workspace, write_mock_executable, KpopSpawn,
    };

    #[test]
    fn kpop_timing_uses_kpop_label_not_implement() {
        let (root, home, workspace) = test_home_workspace();
        let path = bin_path_with_fake_kiss(&root);
        let mock = root.path().join("mock-kpop-timing");
        write_mock_executable(&mock, &acp_mock_code_streaming_update_js());
        let out = spawn_kpop(&KpopSpawn {
            workspace: &workspace,
            home: &home,
            mock: &mock,
            path_var: &path,
            extra_args: &["--max-loops", "1"],
            request: "investigate",
        });
        assert!(
            out.status.success(),
            "expected kpop success when mock streams only chat: {out:?}"
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("TIMING: "),
            "expected timing summary: {stdout:?}"
        );
        assert!(
            stdout.contains("kpop = "),
            "expected kpop timing label: {stdout:?}"
        );
        assert!(
            !stdout.contains("implement = "),
            "did not expect implement timing label in kpop output: {stdout:?}"
        );
        let run_dir = only_run_dir(&workspace, &home);
        let timing_path = run_dir.join("run_timing.json");
        let timing_text = std::fs::read_to_string(&timing_path).expect("read run_timing.json");
        assert!(
            timing_text.contains("\"implement\": \"kpop\""),
            "expected kpop alias in run_timing.json: {timing_text:?}"
        );
        assert!(
            timing_text.contains("\"implement\":"),
            "expected implement phase bucket to remain present in run_timing.json: {timing_text:?}"
        );
    }

    #[test]
    fn kpop_max_loops_controls_outer_agent_runs() {
        let run = run_malvin_under_script_with_mock(
            &acp_mock_code_streaming_update_js(),
            "kpop --max-loops 1 --max-hypotheses 1 investigate",
            None,
        );
        assert!(
            run.output.status.success(),
            "expected kpop success when mock streams only chat: {0:?}",
            run.output
        );
        let stderr = String::from_utf8_lossy(&run.output.stderr);
        assert!(
            !stderr.contains("unexpected argument '--max-loops'"),
            "--max-loops must be a distinct outer-loop flag: {stderr:?}"
        );
    }

    #[test]
    fn kpop_multiturn_restores_before_each_new_turn() {
        let (out, _root, workspace) =
            run_kpop_multiturn_investigate(&acp_mock_kpop_tamper_then_restore_js());
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !combined.contains("ABORT:"),
            "kpop should restore protected files before each prompt: {combined:?}"
        );
        assert_eq!(
            std::fs::read_to_string(workspace.join(".kissconfig")).expect("read kissconfig"),
            "k = 1\n",
            "kpop should restore kissconfig before each prompt: {combined:?}"
        );
        assert_eq!(
            std::fs::read_to_string(workspace.join(".gitignore")).expect("read gitignore"),
            "g = 1\n",
            "kpop should restore gitignore before each prompt: {combined:?}"
        );
    }

    #[test]
    fn do_pty_strips_bold_markers_without_global_no_markdown() {
        let out = run_do_under_script(&[]);
        assert!(
            out.status.success(),
            "expected successful do run under PTY: {out:?}"
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            !stdout.contains("**boldline**"),
            "expected do TTY stdout to render markdown (consume bold markers): {stdout:?}"
        );
        assert!(
            stdout.contains("boldline"),
            "expected bold text content on do TTY stdout: {stdout:?}"
        );
        assert!(
            !stdout.contains("\"jsonrpc\""),
            "stdout leaked JSON-RPC protocol lines: {stdout:?}"
        );
    }

    #[test]
    fn do_pty_preserves_bold_markers_with_global_no_markdown() {
        let out = run_do_under_script(&["--no-markdown"]);
        assert!(
            out.status.success(),
            "expected successful do run under PTY: {out:?}"
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("**boldline**"),
            "expected global --no-markdown to leave do stdout plain: {stdout:?}"
        );
        assert!(
            !stdout.contains("\"jsonrpc\""),
            "stdout leaked JSON-RPC protocol lines: {stdout:?}"
        );
    }
}
