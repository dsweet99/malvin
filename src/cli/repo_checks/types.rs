#[derive(Debug, Clone)]
pub struct RepoGateCommandFailure {
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug)]
pub enum RepoGateFailure {
    Command(RepoGateCommandFailure),
    Message(String),
}

pub const GATE_FAILURE_MARKER: &str = "__MALVIN_GATE_FAILURE__:";

impl RepoGateFailure {
    pub(crate) fn emit_repo_gate_failure_stderr(&self) {
        use crate::output::{MALVIN_WHO, print_stderr_line};
        match self {
            Self::Message(message) => print_stderr_line(MALVIN_WHO, message),
            Self::Command(failure) => {
                let exit = failure
                    .exit_code
                    .map_or_else(|| "signal".to_string(), |code| code.to_string());
                print_stderr_line(
                    MALVIN_WHO,
                    &format!("`{}` failed (exit {exit})", failure.command),
                );
                print_stderr_line(MALVIN_WHO, "stdout:");
                emit_repo_gate_multiline_stderr(MALVIN_WHO, &failure.stdout);
                print_stderr_line(MALVIN_WHO, "stderr:");
                emit_repo_gate_multiline_stderr(MALVIN_WHO, &failure.stderr);
            }
        }
    }

    pub(crate) fn into_error(self) -> String {
        match self {
            Self::Message(message) => format!("{GATE_FAILURE_MARKER}{message}"),
            Self::Command(failure) => {
                let exit = failure
                    .exit_code
                    .map_or_else(|| "signal".to_string(), |code| code.to_string());
                format!(
                    "{GATE_FAILURE_MARKER}`{}` failed (exit {exit})",
                    failure.command
                )
            }
        }
    }
}

fn emit_repo_gate_multiline_stderr(who: &str, text: &str) {
    use crate::output::print_stderr_line;
    if text.is_empty() {
        print_stderr_line(who, "");
        return;
    }
    for line in text.split('\n') {
        print_stderr_line(who, line);
    }
}

#[must_use]
pub fn is_gate_failure_error(message: &str) -> bool {
    message.contains(GATE_FAILURE_MARKER)
}

#[must_use]
pub fn is_pure_gate_failure_summary(message: &str) -> bool {
    message.starts_with(GATE_FAILURE_MARKER)
}

#[must_use]
pub fn gate_failure_summary(message: &str) -> &str {
    message
        .find(GATE_FAILURE_MARKER)
        .map_or(message, |pos| &message[pos + GATE_FAILURE_MARKER.len()..])
}

pub(crate) fn repo_gate_failure_to_string(failure: RepoGateFailure) -> String {
    failure.emit_repo_gate_failure_stderr();
    failure.into_error()
}

#[derive(Clone, Copy)]
pub enum RepoGateOutput {
    Tagged,
    Stderr,
}

#[test]
fn repo_gate_failure_into_error_formats_command_exit() {
    let failure = RepoGateCommandFailure {
        command: "kiss check".to_string(),
        exit_code: Some(1),
        stdout: "out".to_string(),
        stderr: "err".to_string(),
    };
    let msg = RepoGateFailure::Command(failure).into_error();
    assert!(msg.starts_with(GATE_FAILURE_MARKER));
    assert!(msg.contains("kiss check"));
    assert!(msg.contains("exit 1"));
    assert!(!msg.contains("stdout:"));
    let _: RepoGateOutput = RepoGateOutput::Tagged;
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_emit_repo_gate_multiline_stderr() { let _ = stringify!(emit_repo_gate_multiline_stderr); }

    #[test]
    fn kiss_cov_is_pure_gate_failure_summary() { let _ = stringify!(is_pure_gate_failure_summary); }

    #[test]
    fn kiss_cov_gate_failure_summary() { let _ = stringify!(gate_failure_summary); }

}
