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

impl RepoGateFailure {
    pub(crate) fn into_error(self) -> String {
        match self {
            Self::Message(message) => message,
            Self::Command(failure) => {
                let exit = failure
                    .exit_code
                    .map_or_else(|| "signal".to_string(), |code| code.to_string());
                format!(
                    "`{}` failed (exit {}):\nstdout:\n{}\nstderr:\n{}",
                    failure.command, exit, failure.stdout, failure.stderr
                )
            }
        }
    }
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
    assert!(msg.contains("kiss check"));
    assert!(msg.contains("exit 1"));
    let _: RepoGateOutput = RepoGateOutput::Tagged;
}
