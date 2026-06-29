//! Transcript–workspace fork state (see `concepts.md` §2).

use std::hash::{Hash, Hasher};
use std::path::Path;

/// Paired checkpoint of transcript length and workspace manifest at a gate-attempt boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForkState {
    pub message_checkpoint_len: usize,
    pub workspace_manifest_hash: String,
}

impl ForkState {
    #[must_use]
    pub fn capture(cwd: &Path, message_checkpoint_len: usize) -> Self {
        Self {
            message_checkpoint_len,
            workspace_manifest_hash: workspace_manifest_hash(cwd),
        }
    }

    #[must_use]
    pub const fn transcript_matches(&self, current_len: usize) -> bool {
        self.message_checkpoint_len == current_len
    }

    #[must_use]
    pub fn workspace_matches(&self, current_hash: &str) -> bool {
        self.workspace_manifest_hash == current_hash
    }

    #[must_use]
    pub fn is_diverged(&self, current_len: usize, current_hash: &str) -> bool {
        !self.transcript_matches(current_len) || !self.workspace_matches(current_hash)
    }
}

impl From<ForkState> for (usize, String) {
    fn from(state: ForkState) -> Self {
        (state.message_checkpoint_len, state.workspace_manifest_hash)
    }
}

impl From<(usize, String)> for ForkState {
    fn from((message_checkpoint_len, workspace_manifest_hash): (usize, String)) -> Self {
        Self {
            message_checkpoint_len,
            workspace_manifest_hash,
        }
    }
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

#[cfg(test)]
#[path = "fork_state_tests.rs"]
mod fork_state_tests;
