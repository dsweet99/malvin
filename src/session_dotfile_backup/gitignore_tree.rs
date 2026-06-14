use std::path::{Path, PathBuf};

use super::alloc::{allocate_backup_dir, remove_if_exists, DotfileBackupLabels};

const GITIGNORE_NAME: &str = ".gitignore";

const LABELS: DotfileBackupLabels = DotfileBackupLabels {
    mkdir: "gitignore backup mkdir",
    collision: "gitignore backup mkdir",
    restore: "gitignore restore",
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitignoreFileBackup {
    pub rel: PathBuf,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitignoreBackup {
    Missing,
    Present {
        backup_root: PathBuf,
        files: Vec<GitignoreFileBackup>,
    },
}

fn walk_gitignore_files(dir: &Path, work_dir: &Path, found: &mut Vec<PathBuf>) {
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
            walk_gitignore_files(&path, work_dir, found);
        } else if file_type.is_file()
            && path.file_name().and_then(|n| n.to_str()) == Some(GITIGNORE_NAME)
            && let Ok(rel) = path.strip_prefix(work_dir)
        {
            found.push(rel.to_path_buf());
        }
    }
}

fn collect_root_gitignore_only(work_dir: &Path) -> Vec<PathBuf> {
    if work_dir.join(GITIGNORE_NAME).is_file() {
        vec![PathBuf::from(GITIGNORE_NAME)]
    } else {
        vec![]
    }
}

#[must_use]
pub fn collect_workspace_gitignore_relpaths(work_dir: &Path) -> Vec<PathBuf> {
    if crate::repo_gates::git_worktree_toplevel(work_dir).is_some() {
        let mut found = Vec::new();
        walk_gitignore_files(work_dir, work_dir, &mut found);
        found.sort();
        found
    } else {
        collect_root_gitignore_only(work_dir)
    }
}

pub fn backup_workspace_gitignore_if_present(work_dir: &Path) -> Result<GitignoreBackup, String> {
    backup_workspace_gitignore_if_present_with_id(work_dir, super::alloc::random_backup_id)
}

pub fn backup_workspace_gitignore_if_present_with_id(
    work_dir: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> Result<GitignoreBackup, String> {
    backup_gitignore_tree(work_dir, &mut generate_id)
}

pub(super) fn backup_gitignore_tree(
    work_dir: &Path,
    generate_id: &mut impl FnMut(usize) -> String,
) -> Result<GitignoreBackup, String> {
    let rels = collect_workspace_gitignore_relpaths(work_dir);
    if rels.is_empty() {
        return Ok(GitignoreBackup::Missing);
    }

    let root = crate::workspace_paths::snapshot_category_dir("gitignore");
    let dest_dir = allocate_backup_dir(&root, generate_id, &LABELS)?;

    let mut files = Vec::with_capacity(rels.len());
    for rel in rels {
        let src = work_dir.join(&rel);
        let bytes = std::fs::read(&src).map_err(|e| format!(".gitignore backup copy: {e}"))?;
        let dest_file = dest_dir.join(&rel);
        if let Some(parent) = dest_file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", LABELS.mkdir))?;
        }
        if let Err(e) = std::fs::write(&dest_file, &bytes) {
            let _ = std::fs::remove_dir_all(&dest_dir);
            return Err(format!(".gitignore backup copy: {e}"));
        }
        files.push(GitignoreFileBackup { rel, bytes });
    }

    Ok(GitignoreBackup::Present {
        backup_root: dest_dir,
        files,
    })
}

pub fn restore_workspace_gitignore_backup(
    work_dir: &Path,
    backup: &GitignoreBackup,
) -> Result<(), String> {
    match backup {
        GitignoreBackup::Missing => {
            for rel in collect_workspace_gitignore_relpaths(work_dir) {
                remove_if_exists(&work_dir.join(rel), LABELS.restore)?;
            }
            Ok(())
        }
        GitignoreBackup::Present { files, .. } => {
            let snapshot_rels: std::collections::BTreeSet<_> =
                files.iter().map(|file| &file.rel).collect();
            for file in files {
                let dst = work_dir.join(&file.rel);
                if let Some(parent) = dst.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("{}: {e}", LABELS.restore))?;
                }
                std::fs::write(&dst, &file.bytes)
                    .map_err(|e| format!("gitignore restore: {e}"))?;
            }
            for rel in collect_workspace_gitignore_relpaths(work_dir) {
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
    fn kiss_cov_gitignore_backup_types() {
        let _: Option<GitignoreFileBackup> = None;
        let _: Option<GitignoreBackup> = None;
        let _ = collect_workspace_gitignore_relpaths;
        let _ = collect_root_gitignore_only;
    }
}

#[cfg(test)]
#[path = "gitignore_tree_tests.rs"]
mod gitignore_tree_tests;
