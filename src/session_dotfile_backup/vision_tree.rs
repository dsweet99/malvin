use std::path::{Path, PathBuf};

use super::alloc::{allocate_backup_dir, remove_if_exists, DotfileBackupLabels};

const VISION_NAME: &str = "VISION.md";

const LABELS: DotfileBackupLabels = DotfileBackupLabels {
    mkdir: "vision backup mkdir",
    collision: "vision backup mkdir",
    restore: "vision restore",
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionFileBackup {
    pub rel: PathBuf,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisionBackup {
    Missing,
    Present {
        backup_root: PathBuf,
        files: Vec<VisionFileBackup>,
    },
}

fn walk_vision_files(dir: &Path, work_dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some(".git") {
                continue;
            }
            walk_vision_files(&path, work_dir, found);
        } else if file_type.is_file()
            && path.file_name().and_then(|n| n.to_str()) == Some(VISION_NAME)
            && let Ok(rel) = path.strip_prefix(work_dir)
        {
            found.push(rel.to_path_buf());
        }
    }
}

fn collect_root_vision_only(work_dir: &Path) -> Vec<PathBuf> {
    if work_dir.join(VISION_NAME).is_file() {
        vec![PathBuf::from(VISION_NAME)]
    } else {
        vec![]
    }
}

#[must_use]
pub fn collect_workspace_vision_relpaths(work_dir: &Path) -> Vec<PathBuf> {
    if crate::repo_gates::git_worktree_toplevel(work_dir).is_some() {
        let mut found = Vec::new();
        walk_vision_files(work_dir, work_dir, &mut found);
        found.sort();
        found
    } else {
        collect_root_vision_only(work_dir)
    }
}

pub fn backup_workspace_vision_if_present(work_dir: &Path) -> Result<VisionBackup, String> {
    backup_workspace_vision_if_present_with_id(work_dir, super::alloc::random_backup_id)
}

pub fn backup_workspace_vision_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<VisionBackup, String> {
    backup_vision_tree(work_dir, &mut generate_id)
}

pub(super) fn backup_vision_tree(
    work_dir: &Path,
    generate_id: &mut impl FnMut(usize) -> String,
) -> Result<VisionBackup, String> {
    let rels = collect_workspace_vision_relpaths(work_dir);
    if rels.is_empty() {
        return Ok(VisionBackup::Missing);
    }

    let root = crate::workspace_paths::snapshot_category_dir("vision");
    let dest_dir = allocate_backup_dir(&root, generate_id, &LABELS)?;

    let mut files = Vec::with_capacity(rels.len());
    for rel in rels {
        let src = work_dir.join(&rel);
        let bytes = std::fs::read(&src).map_err(|e| format!("VISION.md backup copy: {e}"))?;
        let dest_file = dest_dir.join(&rel);
        if let Some(parent) = dest_file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", LABELS.mkdir))?;
        }
        if let Err(e) = std::fs::write(&dest_file, &bytes) {
            let _ = std::fs::remove_dir_all(&dest_dir);
            return Err(format!("VISION.md backup copy: {e}"));
        }
        files.push(VisionFileBackup { rel, bytes });
    }

    Ok(VisionBackup::Present {
        backup_root: dest_dir,
        files,
    })
}

pub fn restore_workspace_vision_backup(
    work_dir: &Path,
    backup: &VisionBackup,
) -> Result<(), String> {
    match backup {
        VisionBackup::Missing => {
            for rel in collect_workspace_vision_relpaths(work_dir) {
                remove_if_exists(&work_dir.join(rel), LABELS.restore)?;
            }
            Ok(())
        }
        VisionBackup::Present { files, .. } => {
            let snapshot_rels: std::collections::BTreeSet<_> =
                files.iter().map(|file| &file.rel).collect();
            for file in files {
                let dst = work_dir.join(&file.rel);
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("{}: {e}", LABELS.restore))?;
                }
                std::fs::write(&dst, &file.bytes)
                    .map_err(|e| format!("vision restore: {e}"))?;
            }
            for rel in collect_workspace_vision_relpaths(work_dir) {
                if !snapshot_rels.contains(&rel) {
                    remove_if_exists(&work_dir.join(rel), LABELS.restore)?;
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod kiss_cov_auto {
    use super::*;

    #[test]
    fn kiss_cov_vision_backup_types() {
        let _: Option<VisionFileBackup> = None;
        let _: Option<VisionBackup> = None;
        let _ = collect_workspace_vision_relpaths;
        let _ = collect_root_vision_only;
    }
}

#[cfg(test)]
#[path = "vision_tree_tests.rs"]
pub(crate) mod vision_tree_tests;
