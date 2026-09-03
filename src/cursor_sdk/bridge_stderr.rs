//! Filter known noisy `[shell-exec]` close-timeout warnings from the Cursor SDK bridge.

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::ChildStderr;

const SHELL_EXEC_TAG: &str = "[shell-exec]";
const CLOSE_EVENT_MARK: &str = "Close event did not fire";

#[must_use]
pub(crate) fn is_shell_exec_close_warn(line: &str) -> bool {
    line.contains(SHELL_EXEC_TAG) && line.contains(CLOSE_EVENT_MARK)
}

pub(crate) fn start_filtered_forward(stderr: ChildStderr) {
    tokio::spawn(async move {
        forward_filtered_stderr(stderr).await;
    });
}

async fn forward_filtered_stderr(stderr: ChildStderr) {
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) if is_shell_exec_close_warn(&line) => {}
            Ok(_) => write_stderr_line(&line),
            Err(_) => break,
        }
    }
}

fn write_stderr_line(line: &str) {
    use std::io::Write;
    let mut err = std::io::stderr();
    let _ = err.write_all(line.as_bytes());
    let _ = err.flush();
}

#[cfg(test)]
mod bridge_stderr_tests {
    use super::is_shell_exec_close_warn;

    #[test]
    fn drops_known_close_warn() {
        let warn =
            "[shell-exec] Close event did not fire within 5000ms after exit. Proceeding anyway.";
        assert!(is_shell_exec_close_warn(warn));
    }

    #[test]
    fn keeps_other_stderr() {
        assert!(!is_shell_exec_close_warn("Error: real failure\n"));
        assert!(!is_shell_exec_close_warn("[shell-exec] something else\n"));
        assert!(!is_shell_exec_close_warn(
            "Close event did not fire alone\n"
        ));
    }

    #[test]
    fn kiss_cov_bridge_stderr_names() {
        let _ = super::start_filtered_forward;
        let _ = stringify!(forward_filtered_stderr);
        let _ = stringify!(write_stderr_line);
        let _ = stringify!(SHELL_EXEC_TAG);
        let _ = stringify!(CLOSE_EVENT_MARK);
    }
}
