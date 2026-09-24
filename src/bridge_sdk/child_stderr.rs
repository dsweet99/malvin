use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout};

use crate::acp::AgentError;

pub(crate) fn start_warning_forward(stderr: ChildStderr) {
    start_warning_forward_filtered(stderr, |_| false);
}

pub(crate) fn start_warning_forward_filtered(
    stderr: ChildStderr,
    drop_line: impl Fn(&str) -> bool + Send + 'static,
) {
    tokio::spawn(async move {
        forward_stderr_as_warnings(stderr, drop_line).await;
    });
}

pub(crate) fn take_stdio_forward_stderr(
    child: &mut Child,
    label: &str,
) -> Result<(ChildStdin, ChildStdout), AgentError> {
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| AgentError(format!("{label} stdin missing")))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AgentError(format!("{label} stdout missing")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AgentError(format!("{label} stderr missing")))?;
    start_warning_forward(stderr);
    Ok((stdin, stdout))
}

async fn forward_stderr_as_warnings(
    stderr: ChildStderr,
    drop_line: impl Fn(&str) -> bool,
) {
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => emit_backend_stderr_warning(&line, &drop_line),
            Err(_) => break,
        }
    }
}

pub(crate) fn emit_backend_stderr_warning(line: &str, drop_line: &impl Fn(&str) -> bool) {
    let Some(payload) = backend_stderr_warning_payload(line) else {
        return;
    };
    if drop_line(payload) {
        return;
    }
    crate::output::print_log_warning(payload);
}

#[must_use]
pub(crate) fn backend_stderr_warning_payload(line: &str) -> Option<&str> {
    let payload = line.trim_end_matches(['\r', '\n']);
    if payload.is_empty() {
        None
    } else {
        Some(payload)
    }
}
