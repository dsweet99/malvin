use std::path::Path;

use super::gate_restore_checks::substantive_check_lines;
use super::{
    DotfileBackupStateRef, GitignoreBackup, MalvinChecksBackup, MalvinConfigWorkspaceBackup,
    SessionDotfileBackups, VisionBackup,
};

pub(crate) const fn slot_deleted(
    anchor: DotfileBackupStateRef<'_>,
    progress: DotfileBackupStateRef<'_>,
) -> bool {
    matches!(anchor, DotfileBackupStateRef::Present(_))
        && matches!(progress, DotfileBackupStateRef::Missing)
}

pub(crate) const fn slot_bytes(value: DotfileBackupStateRef<'_>) -> Option<&[u8]> {
    match value {
        DotfileBackupStateRef::Present(payload) => Some(payload.bytes.as_slice()),
        DotfileBackupStateRef::Missing => None,
    }
}

pub(crate) fn slot_content_regressed(
    anchor: DotfileBackupStateRef<'_>,
    progress: DotfileBackupStateRef<'_>,
) -> bool {
    let (Some(anchor_bytes), Some(progress_bytes)) = (slot_bytes(anchor), slot_bytes(progress))
    else {
        return false;
    };
    anchor_bytes != progress_bytes
}

pub(crate) fn slot_regressed(
    anchor: DotfileBackupStateRef<'_>,
    progress: DotfileBackupStateRef<'_>,
) -> bool {
    slot_deleted(anchor, progress) || slot_content_regressed(anchor, progress)
}

pub(crate) fn checks_lines_are_superset(anchor_bytes: &[u8], progress_bytes: &[u8]) -> bool {
    let anchor_lines = substantive_check_lines(anchor_bytes);
    let progress_lines = substantive_check_lines(progress_bytes);
    anchor_lines
        .iter()
        .all(|line| progress_lines.iter().any(|p| p == line))
}

pub(crate) fn malvin_checks_regressed(
    anchor: &MalvinChecksBackup,
    progress: &MalvinChecksBackup,
) -> bool {
    let anchor_ref = anchor.as_slot_state();
    let progress_ref = progress.as_slot_state();
    if slot_deleted(anchor_ref, progress_ref) {
        return true;
    }
    let (Some(anchor_bytes), Some(progress_bytes)) =
        (slot_bytes(anchor_ref), slot_bytes(progress_ref))
    else {
        return false;
    };
    if anchor_bytes == progress_bytes {
        return false;
    }
    !checks_lines_are_superset(anchor_bytes, progress_bytes)
}

fn config_workspace_regressed(
    anchor: &MalvinConfigWorkspaceBackup,
    progress: &MalvinConfigWorkspaceBackup,
) -> bool {
    slot_regressed(anchor.as_slot_state(), progress.as_slot_state())
}

fn pick_typed<T: Clone>(anchor: &T, progress: &T, regress_probe: fn(&T, &T) -> bool) -> T {
    if regress_probe(anchor, progress) {
        anchor.clone()
    } else {
        progress.clone()
    }
}

pub(crate) fn root_file_bytes_by_name<'a>(
    files: impl IntoIterator<Item = (&'a Path, &'a [u8])>,
    root_name: &str,
) -> Option<&'a [u8]> {
    files
        .into_iter()
        .find(|(rel, _)| rel.as_os_str() == root_name)
        .map(|(_, bytes)| bytes)
}

pub(crate) const fn root_file_deleted_regressed(
    anchor_root: Option<&[u8]>,
    progress_root: Option<&[u8]>,
) -> bool {
    matches!((anchor_root, progress_root), (Some(_), None))
}

pub(crate) fn gitignore_root_bytes(backup: &GitignoreBackup) -> Option<&[u8]> {
    match backup {
        GitignoreBackup::Missing => None,
        GitignoreBackup::Present { files, .. } => root_file_bytes_by_name(
            files.iter().map(|file| (file.rel.as_path(), file.bytes.as_slice())),
            ".gitignore",
        ),
    }
}

fn gitignore_regressed(anchor: &GitignoreBackup, progress: &GitignoreBackup) -> bool {
    root_file_deleted_regressed(gitignore_root_bytes(anchor), gitignore_root_bytes(progress))
}

pub(crate) fn vision_root_bytes(backup: &VisionBackup) -> Option<&[u8]> {
    match backup {
        VisionBackup::Missing => None,
        VisionBackup::Present { files, .. } => root_file_bytes_by_name(
            files.iter().map(|file| (file.rel.as_path(), file.bytes.as_slice())),
            "VISION.md",
        ),
    }
}

fn vision_regressed(anchor: &VisionBackup, progress: &VisionBackup) -> bool {
    root_file_deleted_regressed(vision_root_bytes(anchor), vision_root_bytes(progress))
}

#[must_use]
pub fn merge_for_gate_restore(
    anchor: &SessionDotfileBackups,
    progress: &SessionDotfileBackups,
) -> SessionDotfileBackups {
    SessionDotfileBackups {
        malvin_checks: pick_typed(
            &anchor.malvin_checks,
            &progress.malvin_checks,
            malvin_checks_regressed,
        ),
        gitignore: pick_typed(&anchor.gitignore, &progress.gitignore, gitignore_regressed),
        vision: pick_typed(&anchor.vision, &progress.vision, vision_regressed),
        malvin_config_workspace: pick_typed(
            &anchor.malvin_config_workspace,
            &progress.malvin_config_workspace,
            config_workspace_regressed,
        ),
    }
}

#[must_use]
pub fn merge_and_sanitize_for_gate_restore(
    anchor: &SessionDotfileBackups,
    progress: &SessionDotfileBackups,
    work_dir: &Path,
) -> SessionDotfileBackups {
    let _ = work_dir;
    merge_for_gate_restore(anchor, progress)
}

#[cfg(test)]
#[path = "gate_restore_merge_tests.rs"]
mod gate_restore_merge_tests;
