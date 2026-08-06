//! Paired checkpoint of mini memory state and workspace tree at a gate-attempt boundary.

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

/// Checkpoint of `(history, previous_response)` plus workspace tree bytes and manifest hash.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForkState {
    pub history: String,
    pub previous_response: String,
    pub workspace_manifest_hash: String,
    /// Relative UTF-8 paths → file contents at capture time (excludes `.git`).
    pub workspace_files: BTreeMap<String, Vec<u8>>,
}

impl ForkState {
    #[must_use]
    pub fn capture(cwd: &Path, history: &str, previous_response: &str) -> Self {
        Self {
            history: history.to_string(),
            previous_response: previous_response.to_string(),
            workspace_manifest_hash: workspace_manifest_hash(cwd),
            workspace_files: capture_workspace_files(cwd),
        }
    }

    /// Restore workspace files captured at [`Self::capture`] time.
    ///
    /// # Errors
    ///
    /// Returns an error when the tree cannot be rewritten.
    pub fn restore_workspace(&self, cwd: &Path) -> Result<(), String> {
        restore_workspace_files(cwd, &self.workspace_files)
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

/// Directories excluded from workspace fork snapshots.
///
/// Gate retries only need a rewind of agent-edited source. Walking dependency /
/// build trees (especially `node_modules` / `target`) can hang Mini before the
/// first HTTP turn and blow memory in repos that vendor JS bridges.
fn skip_dir_name(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | "__pycache__"
            | ".pytest_cache"
            | ".ruff_cache"
            | ".mypy_cache"
            | ".tox"
            | ".venv"
            | "venv"
            | "_logs"
            | "_logs-prime"
    )
}

fn capture_workspace_files(cwd: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut files = BTreeMap::new();
    let mut stack = vec![cwd.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if file_type.is_dir() {
                if skip_dir_name(&name) {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if !file_type.is_file() {
                continue;
            }
            let Ok(rel) = path.strip_prefix(cwd) else {
                continue;
            };
            let rel_key = rel.to_string_lossy().replace('\\', "/");
            if let Ok(bytes) = std::fs::read(&path) {
                files.insert(rel_key, bytes);
            }
        }
    }
    files
}

fn restore_workspace_files(
    cwd: &Path,
    snapshot: &BTreeMap<String, Vec<u8>>,
) -> Result<(), String> {
    let current = capture_workspace_files(cwd);
    for rel in current.keys() {
        if snapshot.contains_key(rel) {
            continue;
        }
        let path = join_rel(cwd, rel);
        std::fs::remove_file(&path).map_err(|e| format!("remove {rel}: {e}"))?;
    }
    for (rel, bytes) in snapshot {
        let path = join_rel(cwd, rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("mkdir {}: {e}", parent.display()))?;
        }
        std::fs::write(&path, bytes).map_err(|e| format!("write {rel}: {e}"))?;
    }
    // Best-effort: remove empty directories left behind after deleting new files.
    remove_empty_dirs(cwd);
    Ok(())
}

fn join_rel(cwd: &Path, rel: &str) -> PathBuf {
    let mut out = cwd.to_path_buf();
    for part in rel.split('/') {
        if part.is_empty() || part == "." {
            continue;
        }
        out.push(part);
    }
    out
}

fn remove_empty_dirs(cwd: &Path) {
    let mut dirs: Vec<PathBuf> = Vec::new();
    let mut stack = vec![cwd.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                let name = entry.file_name();
                if skip_dir_name(&name.to_string_lossy()) {
                    continue;
                }
                stack.push(path.clone());
                if path != cwd {
                    dirs.push(path);
                }
            }
        }
    }
    dirs.sort_by_key(|p| std::cmp::Reverse(p.components().count()));
    for dir in dirs {
        let _ = std::fs::remove_dir(dir);
    }
}

#[cfg(test)]
#[path = "fork_state_tests.rs"]
mod fork_state_tests;
