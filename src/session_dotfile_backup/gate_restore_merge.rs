
use std::path::Path;

use super::gate_restore_checks::substantive_check_lines;
use super::{DotfileBackupState, GitignoreBackup, SessionDotfileBackups, VisionBackup};

pub(crate) const fn slot_deleted(anchor: &DotfileBackupState, progress: &DotfileBackupState) -> bool {
    matches!(anchor, DotfileBackupState::Present(_))
        && matches!(progress, DotfileBackupState::Missing)
}

pub(crate) const fn slot_bytes(value: &DotfileBackupState) -> Option<&[u8]> {
    match value {
        DotfileBackupState::Present(payload) => Some(payload.bytes.as_slice()),
        DotfileBackupState::Missing => None,
    }
}

pub(crate) fn slot_content_regressed(anchor: &DotfileBackupState, progress: &DotfileBackupState) -> bool {
    let (Some(anchor_bytes), Some(progress_bytes)) = (slot_bytes(anchor), slot_bytes(progress))
    else {
        return false;
    };
    anchor_bytes != progress_bytes
}

pub(crate) fn slot_regressed(anchor: &DotfileBackupState, progress: &DotfileBackupState) -> bool {
    slot_deleted(anchor, progress) || slot_content_regressed(anchor, progress)
}

pub(crate) fn checks_lines_are_superset(anchor_bytes: &[u8], progress_bytes: &[u8]) -> bool {
    let anchor_lines = substantive_check_lines(anchor_bytes);
    let progress_lines = substantive_check_lines(progress_bytes);
    anchor_lines
        .iter()
        .all(|line| progress_lines.iter().any(|p| p == line))
}

pub(crate) fn malvin_checks_regressed(anchor: &DotfileBackupState, progress: &DotfileBackupState) -> bool {
    if slot_deleted(anchor, progress) {
        return true;
    }
    let (Some(anchor_bytes), Some(progress_bytes)) = (slot_bytes(anchor), slot_bytes(progress))
    else {
        return false;
    };
    if anchor_bytes == progress_bytes {
        return false;
    }
    !checks_lines_are_superset(anchor_bytes, progress_bytes)
}

fn pick_slot(
    anchor: &DotfileBackupState,
    progress: &DotfileBackupState,
    regress_probe: fn(&DotfileBackupState, &DotfileBackupState) -> bool,
    prefer_progress: fn(&DotfileBackupState, &DotfileBackupState) -> bool,
) -> DotfileBackupState {
    if prefer_progress(anchor, progress) {
        return progress.clone();
    }
    if regress_probe(anchor, progress) {
        return anchor.clone();
    }
    progress.clone()
}

pub(crate) fn gitignore_root_bytes(backup: &GitignoreBackup) -> Option<&[u8]> {
    match backup {
        GitignoreBackup::Missing => None,
        GitignoreBackup::Present { files, .. } => files
            .iter()
            .find(|file| file.rel.as_os_str() == ".gitignore")
            .map(|file| file.bytes.as_slice()),
    }
}

fn gitignore_regressed(anchor: &GitignoreBackup, progress: &GitignoreBackup) -> bool {
    matches!(
        (gitignore_root_bytes(anchor), gitignore_root_bytes(progress)),
        (Some(_), None)
    )
}

fn pick_gitignore(anchor: &GitignoreBackup, progress: &GitignoreBackup) -> GitignoreBackup {
    if gitignore_regressed(anchor, progress) {
        anchor.clone()
    } else {
        progress.clone()
    }
}

pub(crate) fn vision_root_bytes(backup: &VisionBackup) -> Option<&[u8]> {
    match backup {
        VisionBackup::Missing => None,
        VisionBackup::Present { files, .. } => files
            .iter()
            .find(|file| file.rel.as_os_str() == "VISION.md")
            .map(|file| file.bytes.as_slice()),
    }
}

fn vision_regressed(anchor: &VisionBackup, progress: &VisionBackup) -> bool {
    matches!(
        (vision_root_bytes(anchor), vision_root_bytes(progress)),
        (Some(_), None)
    )
}

fn pick_vision(anchor: &VisionBackup, progress: &VisionBackup) -> VisionBackup {
    if vision_regressed(anchor, progress) {
        anchor.clone()
    } else {
        progress.clone()
    }
}

#[must_use]
pub fn merge_for_gate_restore(
    anchor: &SessionDotfileBackups,
    progress: &SessionDotfileBackups,
) -> SessionDotfileBackups {
    SessionDotfileBackups {
        malvin_checks: pick_slot(
            &anchor.malvin_checks,
            &progress.malvin_checks,
            malvin_checks_regressed,
            |_, _| false,
        ),
        gitignore: pick_gitignore(&anchor.gitignore, &progress.gitignore),
        vision: pick_vision(&anchor.vision, &progress.vision),
        malvin_config_workspace: pick_slot(
            &anchor.malvin_config_workspace,
            &progress.malvin_config_workspace,
            slot_regressed,
            |_, _| false,
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
