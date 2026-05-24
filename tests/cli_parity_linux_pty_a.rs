mod common;

#[cfg(all(unix, target_os = "linux"))]
mod linux_pty {
    use crate::common::{
        acp_mock_code_streaming_long_bold_markdown_js, acp_mock_code_streaming_rich_markdown_js,
        assert_markdown_stdout_and_logs, only_run_dir, read_all_logs,
        run_code_max_loops_zero_under_script, run_kpop_catchup_under_script,
        run_malvin_under_script_with_mock,
    };

    #[test]
    fn code_pty_markdown_strips_bold_markers_without_no_markdown() {
        let out = run_code_max_loops_zero_under_script(&[]);
        assert!(
            !out.status.success(),
            "expected max-loops failure exit from script -e: {out:?}"
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            !stdout.contains("**boldline**"),
            "expected termimad to consume ** markers on TTY stdout: {stdout:?}"
        );
        assert!(
            stdout.contains("\x1b[1m"),
            "expected termimad bold ANSI on TTY stdout: {stdout:?}"
        );
        assert!(
            !stdout.contains("\"jsonrpc\""),
            "stdout leaked JSON-RPC protocol lines: {stdout:?}"
        );
    }

    #[test]
    fn code_pty_no_markdown_preserves_bold_markers() {
        let out = run_code_max_loops_zero_under_script(&["--no-markdown"]);
        assert!(
            !out.status.success(),
            "expected max-loops failure exit from script -e: {out:?}"
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("**boldline**"),
            "expected plain stdout to preserve markdown markers: {stdout:?}"
        );
        assert!(
            !stdout.contains("\"jsonrpc\""),
            "stdout leaked JSON-RPC protocol lines: {stdout:?}"
        );
    }

    #[test]
    fn code_pty_no_color_disables_markdown_styling() {
        let out = run_code_max_loops_zero_under_script(&["--no-color"]);
        assert!(
            !out.status.success(),
            "expected max-loops failure exit from script -e: {out:?}"
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("**boldline**"),
            "expected --no-color to leave markdown markers plain on TTY stdout: {stdout:?}"
        );
        assert!(
            !stdout.contains("\x1b[1m"),
            "expected --no-color to suppress ANSI styling on TTY stdout: {stdout:?}"
        );
        assert!(
            !stdout.contains("\"jsonrpc\""),
            "stdout leaked JSON-RPC protocol lines: {stdout:?}"
        );
    }

    #[test]
    fn code_stdout_markdown_styles_stdout_but_logs_stay_raw() {
        let run = run_malvin_under_script_with_mock(
            &acp_mock_code_streaming_rich_markdown_js(),
            "code --trust-the-plan --no-learn --max-loops 0 ship",
            None,
        );
        assert_markdown_stdout_and_logs(&run, "expected max-loops failure exit from script -e");
    }

    #[test]
    fn kpop_stdout_markdown_styles_stdout_but_logs_stay_raw() {
        let run = run_malvin_under_script_with_mock(
            &acp_mock_code_streaming_rich_markdown_js(),
            "kpop --no-learn --max-hypotheses 50 investigate",
            None,
        );
        assert_markdown_stdout_and_logs(&run, "expected kpop catch-up failure exit from script -e");
    }

    #[test]
    fn code_stdout_markdown_wrap_keeps_long_bold_span_styled() {
        let run = run_malvin_under_script_with_mock(
            &acp_mock_code_streaming_long_bold_markdown_js(),
            "code --trust-the-plan --no-learn --max-loops 0 ship",
            Some("40"),
        );
        assert!(
            !run.output.status.success(),
            "expected max-loops failure exit from script -e: {:?}",
            run.output
        );
        let stdout = String::from_utf8_lossy(&run.output.stdout);
        assert!(
            stdout.contains("\x1b[1m"),
            "expected bold ANSI on wrapped TTY stdout: {stdout:?}"
        );
        assert!(
            !stdout.contains("**wrap-bold-xyz"),
            "expected wrapped stdout to avoid leaking opening bold markers: {stdout:?}"
        );
        assert!(
            !stdout.contains("wrap-bold-xyz**"),
            "expected wrapped stdout to avoid leaking closing bold markers: {stdout:?}"
        );
        let run_dir = only_run_dir(&run.workspace);
        let logs = read_all_logs(&run_dir);
        assert!(
            logs.contains("**wrap-bold-xyz wrap-bold-xyz"),
            "expected raw wrapped-bold markdown in logs: {logs:?}"
        );
        assert!(
            !logs.contains("\x1b[1m"),
            "run logs must stay raw without ANSI styling: {logs:?}"
        );
    }

    #[test]
    fn kpop_pty_markdown_strips_bold_markers_without_no_markdown() {
        let out = run_kpop_catchup_under_script(&[]);
        assert!(
            !out.status.success(),
            "expected kpop catch-up failure exit from script -e: {out:?}"
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            !stdout.contains("**boldline**"),
            "expected termimad to consume ** markers on TTY stdout: {stdout:?}"
        );
        assert!(
            stdout.contains("\x1b[1m"),
            "expected termimad bold ANSI on TTY stdout: {stdout:?}"
        );
        assert!(
            !stdout.contains("\"jsonrpc\""),
            "stdout leaked JSON-RPC protocol lines: {stdout:?}"
        );
    }

    #[test]
    fn kpop_pty_no_markdown_preserves_bold_markers() {
        let out = run_kpop_catchup_under_script(&["--no-markdown"]);
        assert!(
            !out.status.success(),
            "expected kpop catch-up failure exit from script -e: {out:?}"
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("**boldline**"),
            "expected plain stdout to preserve markdown markers: {stdout:?}"
        );
        assert!(
            !stdout.contains("\"jsonrpc\""),
            "stdout leaked JSON-RPC protocol lines: {stdout:?}"
        );
    }
}
