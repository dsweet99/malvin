use std::path::{Path, PathBuf};
use std::process::Command;

use super::{MALVIN_CHECKS_REL, MALVIN_DIR};

#[must_use]
pub fn malvin_data_root(work_dir: &Path) -> PathBuf {
    git_worktree_toplevel(work_dir).unwrap_or_else(|| crate::user_home_dir().join(MALVIN_DIR))
}

fn malvin_layout_dir(work_dir: &Path) -> PathBuf {
    if git_worktree_toplevel(work_dir).is_some() {
        malvin_data_root(work_dir).join(MALVIN_DIR)
    } else {
        malvin_data_root(work_dir)
    }
}

#[must_use]
pub fn malvin_checks_path(work_dir: &Path) -> PathBuf {
    malvin_layout_dir(work_dir).join("checks")
}

#[must_use]
pub fn legacy_malvin_checks_path(work_dir: &Path) -> PathBuf {
    work_dir.join(MALVIN_CHECKS_REL)
}

#[must_use]
pub fn resolve_malvin_checks_path(work_dir: &Path) -> PathBuf {
    let primary = malvin_checks_path(work_dir);
    if primary.is_file() {
        return primary;
    }
    let legacy = legacy_malvin_checks_path(work_dir);
    if legacy.is_file() {
        return legacy;
    }
    primary
}

#[must_use]
pub fn malvin_acp_spawn_chamber_dir(work_dir: &Path) -> PathBuf {
    malvin_layout_dir(work_dir).join("acp_spawn")
}

#[must_use]
pub fn git_worktree_toplevel(work_dir: &Path) -> Option<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(work_dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        return None;
    }
    Some(PathBuf::from(root))
}
