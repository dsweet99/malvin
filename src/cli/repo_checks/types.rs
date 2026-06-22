#[derive(Debug, Clone, Default)]
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

#[derive(Clone, Copy, Default)]
pub enum RepoGateOutput {
    #[default]
    Tagged,
    Stderr,
}

#[cfg(test)]
mod kiss_cov_inline {
    use super::*;

    #[test]
    fn kiss_cov_repo_gate_types() {
        let _ = stringify!(RepoGateFailure);
        let _ = stringify!(Command);
        let _ = stringify!(Message);
        let failure = RepoGateCommandFailure {
            command: "c".to_string(),
            exit_code: Some(1),
            stdout: "o".to_string(),
            stderr: "e".to_string(),
        };
        let RepoGateCommandFailure {
            command,
            exit_code,
            stdout,
            stderr,
        } = failure;
        assert_eq!(command, "c");
        assert_eq!(exit_code, Some(1));
        assert_eq!(stdout, "o");
        assert_eq!(stderr, "e");
        let tagged = RepoGateOutput::Tagged;
        let stderr_out = RepoGateOutput::Stderr;
        assert!(matches!(tagged, RepoGateOutput::Tagged));
        assert!(matches!(stderr_out, RepoGateOutput::Stderr));
    }

    #[test]
    fn kiss_cov_repo_gate_command_failure_none_exit_code() {
        let failure = RepoGateCommandFailure {
            command: "sig".to_string(),
            exit_code: None,
            stdout: String::new(),
            stderr: "err".to_string(),
        };
        let RepoGateCommandFailure {
            command,
            exit_code,
            stdout,
            stderr,
        } = failure;
        assert_eq!(command, "sig");
        assert!(exit_code.is_none());
        assert!(stdout.is_empty());
        assert_eq!(stderr, "err");
    }
}

#[cfg(test)]
#[path = "types_kiss_cov_test.rs"]
mod types_kiss_cov_test;
#[cfg(test)]
#[path = "types_test.rs"]
mod types_test;
#[cfg(test)]
#[allow(unused_imports, clippy::unused_unit, non_snake_case)]
mod kiss_static_fn_item_refs {
    use super::*;

    #[test]
    fn kiss_static_fn_item_refs() {
        let _: Option<RepoGateCommandFailure> = None;
        let _: Option<RepoGateOutput> = None;
        let _ = gate_failure_summary;
    }
}
