use std::path::{Path, PathBuf};
use std::process::Command;

use super::{MALVIN_CHECKS_LEGACY_REL, MALVIN_CHECKS_REL, MALVIN_DIR};

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
    malvin_layout_dir(work_dir).join("gates")
}

#[must_use]
pub fn legacy_malvin_checks_path(work_dir: &Path) -> PathBuf {
    work_dir.join(MALVIN_CHECKS_LEGACY_REL)
}

#[must_use]
pub fn resolve_malvin_checks_path(work_dir: &Path) -> PathBuf {
    let primary = malvin_checks_path(work_dir);
    if primary.is_file() {
        return primary;
    }
    let cwd_gates = work_dir.join(MALVIN_CHECKS_REL);
    if cwd_gates.is_file() {
        return cwd_gates;
    }
    let layout_legacy = primary.with_file_name("checks");
    if layout_legacy.is_file() {
        return layout_legacy;
    }
    let legacy = legacy_malvin_checks_path(work_dir);
    if legacy.is_file() {
        return legacy;
    }
    primary
}

#[must_use]
pub fn malvin_acp_spawn_chamber_dir(work_dir: &Path) -> PathBuf {
    let home_chamber = malvin_layout_dir(work_dir).join("acp_spawn");
    if directory_is_writable(home_chamber.parent().unwrap_or(&home_chamber)) {
        home_chamber
    } else {
        work_dir.join(MALVIN_DIR).join("acp_spawn")
    }
}

fn directory_is_writable(dir: &Path) -> bool {
    if std::fs::create_dir_all(dir).is_err() {
        return false;
    }
    let probe = dir.join(format!(".write-probe-{}", std::process::id()));
    let writable = std::fs::write(&probe, []).is_ok();
    let _ = std::fs::remove_file(probe);
    writable
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
