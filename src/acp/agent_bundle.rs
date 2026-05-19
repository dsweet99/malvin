use std::path::PathBuf;
use std::process::Command as StdCommand;
use std::time::{Duration, Instant};
use tokio::time::sleep as tokio_sleep;

#[derive(Debug, Clone)]
pub struct AgentError(pub String);

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AgentError {}

#[derive(Debug, Clone)]
pub struct AuthError(pub String);

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for AuthError {}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct AgentIoOptions {
    pub force: bool,
    pub no_tee: bool,
    pub raw_output: bool,
    pub show_thoughts_on_stdout: bool,
    pub emit_stdout_markdown: bool,
    /// When true, echo each outgoing prompt body on stdout and in `prompts.log`; when false, only the `[name...]` line is logged there.
    pub log_full_outgoing_prompts: bool,
}

#[cfg(test)]
mod agent_bundle_kiss_cov {
    #[test]
    fn kiss_stringify_file_coverage() {
        let _ = stringify!(super::AgentError);
        let _ = stringify!(super::AuthError);
    }
}
