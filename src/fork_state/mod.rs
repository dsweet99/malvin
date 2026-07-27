//! Paired checkpoint of mini memory state and workspace manifest at a gate-attempt boundary.

use std::hash::{Hash, Hasher};
use std::path::Path;

/// Checkpoint of `(history, previous_response)` plus workspace manifest hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForkState {
    pub history: String,
    pub previous_response: String,
    pub workspace_manifest_hash: String,
}

impl ForkState {
    #[must_use]
    pub fn capture(cwd: &Path, history: &str, previous_response: &str) -> Self {
        Self {
            history: history.to_string(),
            previous_response: previous_response.to_string(),
            workspace_manifest_hash: workspace_manifest_hash(cwd),
        }
    }

    #[must_use]
    pub fn memory_matches(&self, history: &str, previous_response: &str) -> bool {
        self.history == history && self.previous_response == previous_response
    }

    #[must_use]
    pub fn workspace_matches(&self, current_hash: &str) -> bool {
        self.workspace_manifest_hash == current_hash
    }

    #[must_use]
    pub fn is_diverged(
        &self,
        history: &str,
        previous_response: &str,
        current_hash: &str,
    ) -> bool {
        !self.memory_matches(history, previous_response) || !self.workspace_matches(current_hash)
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
