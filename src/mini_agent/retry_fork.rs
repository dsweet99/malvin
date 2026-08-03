//! Gate-iteration retry fork ledger (`miniRetryFork` trace events).

use super::fork_state::ForkState;

#[allow(unused_imports)]
pub use super::fork_state::workspace_manifest_hash;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiniRetryStrategy {
    CumulativeTranscript,
    WorkspaceSnapshot,
}

impl MiniRetryStrategy {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CumulativeTranscript => "cumulative-transcript",
            Self::WorkspaceSnapshot => "workspace-snapshot",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForkOutcome {
    Succeeded,
    Failed,
}

impl ForkOutcome {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryForkLedger {
    pub prompt_index: u32,
    pub attempt: u32,
    pub history: String,
    pub previous_response: String,
    pub workspace_manifest_hash: String,
    pub bash_commands: Vec<String>,
    pub outcome: ForkOutcome,
    pub strategy: MiniRetryStrategy,
}

impl RetryForkLedger {
    #[must_use]
    pub fn checkpoint(&self) -> ForkState {
        ForkState {
            history: self.history.clone(),
            previous_response: self.previous_response.clone(),
            workspace_manifest_hash: self.workspace_manifest_hash.clone(),
        }
    }
}

pub fn build_divergence_observation(
    bash_commands: &[String],
    failure_reason: &str,
    manifest_hash: &str,
) -> String {
    let cmds = if bash_commands.is_empty() {
        "(none)".to_string()
    } else {
        bash_commands.join("\n")
    };
    format!(
        "[mini retry divergence]\nworkspace_manifest_hash: {manifest_hash}\ncommands_run:\n{cmds}\nlast_failure:\n{failure_reason}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divergence_observation_empty_commands() {
        let obs = build_divergence_observation(&[], "boom", "git:abc");
        assert!(obs.contains("(none)"));
        assert!(obs.contains("boom"));
        assert!(obs.contains("git:abc"));
    }

    #[test]
    fn ledger_checkpoint_round_trip() {
        let ledger = RetryForkLedger {
            prompt_index: 1,
            attempt: 2,
            history: "h".into(),
            previous_response: "p".into(),
            workspace_manifest_hash: "git:x".into(),
            bash_commands: vec![],
            outcome: ForkOutcome::Failed,
            strategy: MiniRetryStrategy::WorkspaceSnapshot,
        };
        assert_eq!(
            ledger.checkpoint(),
            ForkState {
                history: "h".into(),
                previous_response: "p".into(),
                workspace_manifest_hash: "git:x".into(),
            }
        );
    }
}
