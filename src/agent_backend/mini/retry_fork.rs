//! Gate-iteration retry fork ledger (`miniRetryFork` trace events).

use crate::fork_state::ForkState;

#[allow(unused_imports)]
pub use crate::fork_state::workspace_manifest_hash;

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
    pub message_checkpoint_len: usize,
    pub workspace_manifest_hash: String,
    pub bash_commands: Vec<String>,
    pub outcome: ForkOutcome,
    pub strategy: MiniRetryStrategy,
}

impl RetryForkLedger {
    #[must_use]
    pub fn checkpoint(&self) -> ForkState {
        ForkState {
            message_checkpoint_len: self.message_checkpoint_len,
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
    fn fork_outcome_and_strategy_wire_names() {
        assert_eq!(ForkOutcome::Failed.as_str(), "failed");
        assert_eq!(MiniRetryStrategy::WorkspaceSnapshot.as_str(), "workspace-snapshot");
        let ledger = RetryForkLedger {
            prompt_index: 1,
            attempt: 2,
            message_checkpoint_len: 3,
            workspace_manifest_hash: "h".into(),
            bash_commands: vec!["echo".into()],
            outcome: ForkOutcome::Failed,
            strategy: MiniRetryStrategy::WorkspaceSnapshot,
        };
        assert_eq!(ledger.attempt, 2);
        assert_eq!(
            ledger.checkpoint(),
            ForkState {
                message_checkpoint_len: 3,
                workspace_manifest_hash: "h".into(),
            }
        );
    }

    #[test]
    fn divergence_observation_empty_commands() {
        let obs = build_divergence_observation(&[], "boom", "git:abc");
        assert!(obs.contains("(none)"));
    }
}
