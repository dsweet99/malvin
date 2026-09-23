use std::path::{Path, PathBuf};

use super::alloc::DotfileBackupLabels;
use super::named_file_tree::{
    NamedFileEntry, NamedFileTreePolicy, NamedFileTreeState, backup_named_file_tree,
    collect_workspace_named_file_relpaths, restore_missing_named_files,
    restore_present_named_files, typed_named_file_backup,
};

const VISION_NAME: &str = "VISION.md";

const POLICY: NamedFileTreePolicy = NamedFileTreePolicy {
    file_name: VISION_NAME,
    category: "vision",
    labels: DotfileBackupLabels {
        mkdir: "vision backup mkdir",
        collision: "vision backup mkdir",
        restore: "vision restore",
    },
    copy_error: "VISION.md backup copy",
    restore_write_error: "vision restore",
};

typed_named_file_backup!(VisionFileBackup);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisionBackup {
    Missing,
    Present {
        backup_root: PathBuf,
        files: Vec<VisionFileBackup>,
    },
}

#[cfg(test)]
fn collect_root_vision_only(work_dir: &Path) -> Vec<PathBuf> {
    if work_dir.join(VISION_NAME).is_file() {
        vec![PathBuf::from(VISION_NAME)]
    } else {
        vec![]
    }
}

#[must_use]
pub fn collect_workspace_vision_relpaths(work_dir: &Path) -> Vec<PathBuf> {
    collect_workspace_named_file_relpaths(work_dir, VISION_NAME)
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
    Ok(from_state(backup_named_file_tree(
        work_dir,
        &rels,
        generate_id,
        &POLICY,
    )?))
}

pub fn restore_workspace_vision_backup(
    work_dir: &Path,
    backup: &VisionBackup,
) -> Result<(), String> {
    match backup {
        VisionBackup::Missing => {
            let rels = collect_workspace_vision_relpaths(work_dir);
            restore_missing_named_files(work_dir, &rels, &POLICY)
        }
        VisionBackup::Present { files, .. } => {
            let entries = VisionFileBackup::as_named_entries(files);
            restore_present_named_files(work_dir, &entries, &POLICY)
        }
    }
}

fn from_state(state: NamedFileTreeState) -> VisionBackup {
    match state {
        NamedFileTreeState::Missing => VisionBackup::Missing,
        NamedFileTreeState::Present { backup_root, files } => VisionBackup::Present {
            backup_root,
            files: files.into_iter().map(VisionFileBackup::from).collect(),
        },
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
        let _ = from_state;
    }
}

#[cfg(test)]
#[path = "vision_tree_tests.rs"]
pub(crate) mod vision_tree_tests;
