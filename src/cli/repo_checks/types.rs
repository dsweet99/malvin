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

impl std::fmt::Display for RepoGateCommandFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.exit_code {
            Some(code) => write!(f, "`{}` failed (exit {code})", self.command),
            None => write!(f, "`{}` failed (exit signal)", self.command),
        }
    }
}

impl std::fmt::Display for RepoGateFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Message(message) => f.write_str(message),
            Self::Command(failure) => failure.fmt(f),
        }
    }
}

impl RepoGateFailure {
    pub(crate) fn emit_repo_gate_failure_stderr(&self) {
        use malvin::output::{MALVIN_WHO, print_stderr_line};
        print_stderr_line(MALVIN_WHO, &self.to_string());
        if let Self::Command(failure) = self {
            print_stderr_line(MALVIN_WHO, "stdout:");
            emit_repo_gate_multiline_stderr(MALVIN_WHO, &failure.stdout);
            print_stderr_line(MALVIN_WHO, "stderr:");
            emit_repo_gate_multiline_stderr(MALVIN_WHO, &failure.stderr);
        }
    }
}

fn emit_repo_gate_multiline_stderr(who: &str, text: &str) {
    use malvin::output::print_stderr_line;
    if text.is_empty() {
        print_stderr_line(who, "");
        return;
    }
    for line in text.split('\n') {
        print_stderr_line(who, line);
    }
}

#[derive(Debug)]
pub enum GateRunError {
    Gate(RepoGateFailure),
    Setup(String),
}

impl GateRunError {
    #[must_use]
    pub const fn is_command_failure(&self) -> bool {
        matches!(self, Self::Gate(RepoGateFailure::Command(_)))
    }
}

impl std::fmt::Display for GateRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gate(failure) => failure.fmt(f),
            Self::Setup(message) => f.write_str(message),
        }
    }
}

impl From<String> for GateRunError {
    fn from(message: String) -> Self {
        Self::Setup(message)
    }
}

#[derive(Clone, Copy)]
pub enum RepoGateOutput {
    Tagged,
    #[cfg(test)]
    Stderr,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
