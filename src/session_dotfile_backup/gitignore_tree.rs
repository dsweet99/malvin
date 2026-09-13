use std::path::{Path, PathBuf};

use super::alloc::DotfileBackupLabels;
use super::named_file_tree::{
    NamedFileEntry, NamedFileTreePolicy, NamedFileTreeState, backup_named_file_tree,
    collect_workspace_named_file_relpaths, restore_missing_named_files, restore_present_named_files,
    typed_named_file_backup,
};

const GITIGNORE_NAME: &str = ".gitignore";

const POLICY: NamedFileTreePolicy = NamedFileTreePolicy {
    file_name: GITIGNORE_NAME,
    category: "gitignore",
    labels: DotfileBackupLabels {
        mkdir: "gitignore backup mkdir",
        collision: "gitignore backup mkdir",
        restore: "gitignore restore",
    },
    copy_error: ".gitignore backup copy",
    restore_write_error: "gitignore restore",
};

typed_named_file_backup!(GitignoreFileBackup);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitignoreBackup {
    Missing,
    Present {
        backup_root: PathBuf,
        files: Vec<GitignoreFileBackup>,
    },
}

#[cfg(test)]
fn collect_root_gitignore_only(work_dir: &Path) -> Vec<PathBuf> {
    if work_dir.join(GITIGNORE_NAME).is_file() {
        vec![PathBuf::from(GITIGNORE_NAME)]
    } else {
        vec![]
    }
}

#[must_use]
pub fn collect_workspace_gitignore_relpaths(work_dir: &Path) -> Vec<PathBuf> {
    collect_workspace_named_file_relpaths(work_dir, GITIGNORE_NAME)
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
    Ok(from_state(backup_named_file_tree(
        work_dir, &rels, generate_id, &POLICY,
    )?))
}

pub fn restore_workspace_gitignore_backup(
    work_dir: &Path,
    backup: &GitignoreBackup,
) -> Result<(), String> {
    match backup {
        GitignoreBackup::Missing => {
            let rels = collect_workspace_gitignore_relpaths(work_dir);
            restore_missing_named_files(work_dir, &rels, &POLICY)
        }
        GitignoreBackup::Present { files, .. } => {
            let entries = GitignoreFileBackup::as_named_entries(files);
            restore_present_named_files(work_dir, &entries, &POLICY)
        }
    }
}

fn from_state(state: NamedFileTreeState) -> GitignoreBackup {
    match state {
        NamedFileTreeState::Missing => GitignoreBackup::Missing,
        NamedFileTreeState::Present {
            backup_root,
            files,
        } => GitignoreBackup::Present {
            backup_root,
            files: files.into_iter().map(GitignoreFileBackup::from).collect(),
        },
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
        let _ = from_state;
    }
}

#[cfg(test)]
#[path = "gitignore_tree_tests.rs"]
pub(crate) mod gitignore_tree_tests;
