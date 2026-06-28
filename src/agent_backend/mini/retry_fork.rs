//! Gate-iteration retry fork ledger (`miniRetryFork` trace events).

use std::hash::{Hash, Hasher};
use std::path::Path;

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

/// Best-effort workspace manifest hash from `git status --porcelain` or empty cwd listing.
#[must_use]
pub fn workspace_manifest_hash(cwd: &Path) -> String {
    let git = std::process::Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(cwd)
        .output();
    if let Ok(out) = git {
        if out.status.success() {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            out.stdout.hash(&mut hasher);
            return format!("git:{:x}", hasher.finish());
        }
    }
    let mut names: Vec<String> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(cwd) {
        for entry in entries.flatten() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for name in &names {
        name.hash(&mut hasher);
    }
    format!("dir:{:x}", hasher.finish())
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
    fn workspace_manifest_hash_is_stable_for_same_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let a = workspace_manifest_hash(tmp.path());
        let b = workspace_manifest_hash(tmp.path());
        assert_eq!(a, b);
        assert!(!a.is_empty());
    }

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
    }

    #[test]
    fn divergence_observation_empty_commands() {
        let obs = build_divergence_observation(&[], "boom", "git:abc");
        assert!(obs.contains("(none)"));
    }
}
