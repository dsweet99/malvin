use crate::cli::repo_checks::GateRunError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliError {
    GatesFailed(String),
    Other(String),
}

impl CliError {
    #[must_use]
    pub fn from_router_gates(command: &str, failure: &GateRunError) -> Self {
        let text = crate::cli::format_workspace_gate_failure(command, &failure.to_string());
        match failure {
            GateRunError::Gate(_) => Self::GatesFailed(text),
            GateRunError::Setup(_) => Self::Other(text),
        }
    }
}

impl From<String> for CliError {
    fn from(message: String) -> Self {
        Self::Other(message)
    }
}

impl From<&str> for CliError {
    fn from(message: &str) -> Self {
        Self::Other(message.to_string())
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GatesFailed(text) | Self::Other(text) => f.write_str(text),
        }
    }
}
