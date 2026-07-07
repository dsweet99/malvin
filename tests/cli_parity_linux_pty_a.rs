mod common;

#[cfg(all(unix, target_os = "linux"))]
mod linux_pty {
    use crate::common::{
        acp_mock_code_streaming_rich_markdown_js, assert_markdown_stdout_and_logs,
        run_kpop_bold_markdown_under_openpty, run_malvin_under_openpty_with_mock,
    };

    #[test]
    fn kpop_stdout_markdown_styles_stdout_but_logs_stay_raw() {
        let run = run_malvin_under_openpty_with_mock(
            &acp_mock_code_streaming_rich_markdown_js(),
            "kpop --max-loops 1 investigate",
            None,
        );
        assert!(
            run.output.status.success(),
            "expected kpop success when agent streams markdown only: {:?}",
            run.output
        );
        assert_markdown_stdout_and_logs(&run);
    }

    #[test]
    fn kpop_pty_markdown_strips_bold_markers_without_no_markdown() {
        let out = run_kpop_bold_markdown_under_openpty(&[]);
        assert!(
            out.status.success(),
            "expected kpop success when agent streams markdown only: {out:?}"
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
        let out = run_kpop_bold_markdown_under_openpty(&["--no-markdown"]);
        assert!(
            out.status.success(),
            "expected kpop success when agent streams markdown only: {out:?}"
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
