mod common;

#[cfg(all(unix, target_os = "linux"))]
mod linux_pty {
    use crate::common::run_do_under_openpty;

    #[test]
    fn do_pty_strips_bold_markers() {
        let out = run_do_under_openpty(&[]);
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
}
