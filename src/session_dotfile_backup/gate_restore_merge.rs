//! Merge pre-agent anchor with post-agent progress for gate restore.
//!
//! Post-agent snapshots capture agent-created fixes (e.g. new `.kissignore`) but also
//! pathological damage (deleted ignore files, clamp-regenerated `.kissconfig`, bare `kiss`
//! in checks). Gate restore must prefer the anchor when progress regresses a slot.

use std::path::Path;

use super::gate_restore_checks::{is_bare_kiss_checks, is_invalid_bare_kiss_checks, substantive_check_lines};
use super::gate_restore_repair::sanitize_clamp_damaged_dotfiles_in_bundle;
use super::{DotfileBackupState, GitignoreBackup, SessionDotfileBackups, VisionBackup};

pub(super) fn kissconfig_low_coverage_threshold(bytes: &[u8]) -> bool {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return true;
    };
    let Ok(value) = text.parse::<toml::Value>() else {
        return true;
    };
    crate::repo_checks::kissconfig_warn::should_warn_low_test_coverage(&value)
}

pub(crate) const fn slot_deleted(anchor: &DotfileBackupState, progress: &DotfileBackupState) -> bool {
    matches!(anchor, DotfileBackupState::Present(_))
        && matches!(progress, DotfileBackupState::Missing)
}

pub(crate) const fn kissignore_agent_created(
    anchor: &DotfileBackupState,
    progress: &DotfileBackupState,
) -> bool {
    matches!(anchor, DotfileBackupState::Missing)
        && matches!(progress, DotfileBackupState::Present(_))
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

pub(crate) fn kissconfig_threshold_regressed(
    anchor: &DotfileBackupState,
    progress: &DotfileBackupState,
) -> bool {
    let (Some(anchor_bytes), Some(progress_bytes)) = (slot_bytes(anchor), slot_bytes(progress))
    else {
        return false;
    };
    let anchor_low = kissconfig_low_coverage_threshold(anchor_bytes);
    let progress_low = kissconfig_low_coverage_threshold(progress_bytes);
    !anchor_low && progress_low
}

pub(crate) fn kissconfig_repaired_clamp_damage(
    anchor: &DotfileBackupState,
    progress: &DotfileBackupState,
) -> bool {
    let (Some(anchor_bytes), Some(progress_bytes)) = (slot_bytes(anchor), slot_bytes(progress))
    else {
        return false;
    };
    kissconfig_low_coverage_threshold(anchor_bytes)
        && !kissconfig_low_coverage_threshold(progress_bytes)
}

pub(crate) fn malvin_checks_repaired_clamp_damage(
    anchor: &DotfileBackupState,
    progress: &DotfileBackupState,
) -> bool {
    is_invalid_bare_kiss_checks(anchor) && !is_invalid_bare_kiss_checks(progress)
}

pub(crate) fn kissconfig_regressed(anchor: &DotfileBackupState, progress: &DotfileBackupState) -> bool {
    slot_deleted(anchor, progress)
        || slot_content_regressed(anchor, progress)
        || kissconfig_threshold_regressed(anchor, progress)
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
    if is_bare_kiss_checks(progress) && !is_bare_kiss_checks(anchor) {
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

fn gitignore_root_bytes(backup: &GitignoreBackup) -> Option<&[u8]> {
    match backup {
        GitignoreBackup::Missing => None,
        GitignoreBackup::Present { files, .. } => files
            .iter()
            .find(|file| file.rel.as_os_str() == ".gitignore")
            .map(|file| file.bytes.as_slice()),
    }
}

fn gitignore_regressed(anchor: &GitignoreBackup, progress: &GitignoreBackup) -> bool {
    match (gitignore_root_bytes(anchor), gitignore_root_bytes(progress)) {
        (Some(_anchor_bytes), None) => true,
        (Some(anchor_bytes), Some(progress_bytes)) => anchor_bytes != progress_bytes,
        _ => false,
    }
}

fn pick_gitignore(anchor: &GitignoreBackup, progress: &GitignoreBackup) -> GitignoreBackup {
    if gitignore_regressed(anchor, progress) {
        anchor.clone()
    } else {
        progress.clone()
    }
}

fn vision_root_bytes(backup: &VisionBackup) -> Option<&[u8]> {
    match backup {
        VisionBackup::Missing => None,
        VisionBackup::Present { files, .. } => files
            .iter()
            .find(|file| file.rel.as_os_str() == "VISION.md")
            .map(|file| file.bytes.as_slice()),
    }
}

fn vision_regressed(anchor: &VisionBackup, progress: &VisionBackup) -> bool {
    match (vision_root_bytes(anchor), vision_root_bytes(progress)) {
        (Some(_anchor_bytes), None) => true,
        (Some(anchor_bytes), Some(progress_bytes)) => anchor_bytes != progress_bytes,
        _ => false,
    }
}

fn pick_vision(anchor: &VisionBackup, progress: &VisionBackup) -> VisionBackup {
    if vision_regressed(anchor, progress) {
        anchor.clone()
    } else {
        progress.clone()
    }
}

/// Merge anchor (iteration start) with progress (post-agent, pre-restore) for gate restore.
#[must_use]
pub fn merge_for_gate_restore(
    anchor: &SessionDotfileBackups,
    progress: &SessionDotfileBackups,
) -> SessionDotfileBackups {
    SessionDotfileBackups {
        kissconfig: pick_slot(
            &anchor.kissconfig,
            &progress.kissconfig,
            kissconfig_regressed,
            kissconfig_repaired_clamp_damage,
        ),
        malvin_checks: pick_slot(
            &anchor.malvin_checks,
            &progress.malvin_checks,
            malvin_checks_regressed,
            malvin_checks_repaired_clamp_damage,
        ),
        kissignore: pick_slot(
            &anchor.kissignore,
            &progress.kissignore,
            slot_regressed,
            kissignore_agent_created,
        ),
        malvin_config: pick_slot(
            &anchor.malvin_config,
            &progress.malvin_config,
            slot_regressed,
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

/// Merge anchor/progress snapshots and sanitize any remaining clamp damage in the bundle.
#[must_use]
pub fn merge_and_sanitize_for_gate_restore(
    anchor: &SessionDotfileBackups,
    progress: &SessionDotfileBackups,
    work_dir: &Path,
) -> SessionDotfileBackups {
    let mut merged = merge_for_gate_restore(anchor, progress);
    sanitize_clamp_damaged_dotfiles_in_bundle(&mut merged, work_dir);
    merged
}

#[cfg(test)]
#[path = "gate_restore_merge_tests.rs"]
mod gate_restore_merge_tests;
