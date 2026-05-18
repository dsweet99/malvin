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
fn kiss_stringify_repo_checks_types() {
    let _ = stringify!(RepoGateCommandFailure);
    let _ = stringify!(RepoGateFailure);
    let _ = stringify!(RepoGateOutput);
}
