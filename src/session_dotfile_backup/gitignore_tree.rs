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

#[must_use]
pub fn collect_workspace_gitignore_relpaths(work_dir: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    walk_gitignore_files(work_dir, work_dir, &mut found);
    found.sort();
    found
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
mod tests {
    use super::*;
    use crate::test_utils::with_isolated_home;

    #[test]
    fn collect_finds_root_and_nested_gitignore_files() {
        let tmp = tempfile::tempdir().unwrap();
        let work = tmp.path().join("repo");
        std::fs::create_dir_all(work.join("pkg/.cache")).unwrap();
        std::fs::write(work.join(".gitignore"), "root\n").unwrap();
        std::fs::write(work.join("pkg/.gitignore"), "pkg\n").unwrap();
        std::fs::write(work.join("pkg/.cache/.gitignore"), "cache\n").unwrap();

        let rels = collect_workspace_gitignore_relpaths(&work);
        assert_eq!(
            rels,
            vec![
                PathBuf::from(".gitignore"),
                PathBuf::from("pkg/.cache/.gitignore"),
                PathBuf::from("pkg/.gitignore"),
            ]
        );
    }

    #[test]
    fn nested_gitignore_round_trip_restores_tree_and_removes_agent_created_files() {
        with_isolated_home(|work| {
            std::fs::create_dir_all(work.join("pkg")).unwrap();
            std::fs::write(work.join(".gitignore"), "root\n").unwrap();
            std::fs::write(work.join("pkg/.gitignore"), "pkg\n").unwrap();

            let backup = super::backup_workspace_gitignore_if_present_with_id(work, &mut |n| {
                format!("gi{n}")
            })
            .unwrap();
            let GitignoreBackup::Present { backup_root, files } = &backup else {
                panic!("expected gitignore tree backup");
            };
            assert!(backup_root.starts_with(
                crate::workspace_paths::snapshot_category_dir("gitignore")
            ));
            assert_eq!(files.len(), 2);

            std::fs::write(work.join(".gitignore"), "tampered-root\n").unwrap();
            std::fs::write(work.join("pkg/.gitignore"), "tampered-pkg\n").unwrap();
            std::fs::create_dir_all(work.join("new")).unwrap();
            std::fs::write(work.join("new/.gitignore"), "agent-created\n").unwrap();

            restore_workspace_gitignore_backup(work, &backup).unwrap();
            assert_eq!(
                std::fs::read_to_string(work.join(".gitignore")).unwrap(),
                "root\n"
            );
            assert_eq!(
                std::fs::read_to_string(work.join("pkg/.gitignore")).unwrap(),
                "pkg\n"
            );
            assert!(!work.join("new/.gitignore").exists());
        });
    }

    #[test]
    fn poisoned_disk_snapshot_does_not_change_restored_gitignore_content() {
        with_isolated_home(|work| {
            std::fs::write(work.join(".gitignore"), "ORIGINAL\n").unwrap();
            let backup = super::backup_workspace_gitignore_if_present_with_id(work, &mut |n| {
                format!("poison{n}")
            })
            .unwrap();
            let GitignoreBackup::Present { backup_root, .. } = &backup else {
                panic!("expected backup");
            };
            std::fs::write(backup_root.join(".gitignore"), "POISONED\n").unwrap();
            std::fs::write(work.join(".gitignore"), "AGENT\n").unwrap();

            restore_workspace_gitignore_backup(work, &backup).unwrap();
            assert_eq!(
                std::fs::read_to_string(work.join(".gitignore")).unwrap(),
                "ORIGINAL\n"
            );
        });
    }
}
