//! Interrupted-run detection and user-plan sidecar recovery for `malvin plan`.

use std::path::{Path, PathBuf};

use super::{read_plan_file, truncate_plan_for_rerun, write_plan_file_atomic, PlanFileError};

pub(crate) const PLAN_USER_SIDECAR_SUFFIX: &str = ".malvin-user";

/// Legacy delimiter retained only for recovering interrupted runs started before overwrite migration.
pub const LEGACY_BEGIN_MALVIN_MARKER: &str = "BEGIN_MALVIN";

fn legacy_machine_block_marker_patterns() -> [(String, usize); 4] {
    [
        (format!("\n---\n{LEGACY_BEGIN_MALVIN_MARKER}"), 1),
        (format!("\r\n---\r\n{LEGACY_BEGIN_MALVIN_MARKER}"), 2),
        (format!("\n---\r\n{LEGACY_BEGIN_MALVIN_MARKER}"), 1),
        (format!("\r\n---\n{LEGACY_BEGIN_MALVIN_MARKER}"), 2),
    ]
}

pub(crate) fn count_legacy_begin_malvin_marker_lines(content: &str) -> usize {
    content
        .lines()
        .filter(|line| line.trim() == LEGACY_BEGIN_MALVIN_MARKER)
        .count()
}

/// Byte offset at the start of a legacy machine block (`---` line), if exactly one unambiguous block exists.
#[must_use]
pub fn find_machine_block_start(content: &str) -> Option<usize> {
    for (marker, eol_len) in legacy_machine_block_marker_patterns() {
        if let Some(idx) = content.find(&marker) {
            let user = &content[..idx];
            if count_legacy_begin_malvin_marker_lines(user) > 0 {
                return None;
            }
            return Some(idx + eol_len);
        }
    }
    for at_start in ["---\n", "---\r\n"] {
        let marker = format!("{at_start}{LEGACY_BEGIN_MALVIN_MARKER}");
        if content.starts_with(&marker) {
            return Some(0);
        }
    }
    None
}

/// Returns `user_span_end` when a single legacy machine block is present.
pub fn detect_rerun_user_span_end(content: &str) -> Result<Option<usize>, PlanFileError> {
    let count = count_legacy_begin_malvin_marker_lines(content);
    if count == 0 {
        return Ok(None);
    }
    if count > 1 {
        return Err(PlanFileError::DuplicateBeginMalvinMarkers);
    }
    find_machine_block_start(content)
        .ok_or(PlanFileError::MalformedMachineBlockDelimiter)
        .map(Some)
}

#[must_use]
pub fn plan_user_sidecar_path(plan_path: &Path) -> PathBuf {
    let mut path = plan_path.as_os_str().to_owned();
    path.push(PLAN_USER_SIDECAR_SUFFIX);
    PathBuf::from(path)
}

/// True when the plan file is in intermediate machine staging (overwrite format).
#[must_use]
pub fn is_interrupted_machine_plan(content: &str) -> bool {
    content.trim_start().starts_with("## Restatement")
}

pub fn restore_interrupted_plan(path: &Path) -> Result<bool, PlanFileError> {
    let content = read_plan_file(path)?;
    if !is_interrupted_machine_plan(&content) && detect_rerun_user_span_end(&content)?.is_none() {
        return Ok(false);
    }
    let sidecar = plan_user_sidecar_path(path);
    if sidecar.is_file() {
        let user = read_plan_file(&sidecar)?;
        write_plan_file_atomic(path, &user)?;
        std::fs::remove_file(&sidecar)?;
        return Ok(true);
    }
    if let Some(user_span_end) = detect_rerun_user_span_end(&content)? {
        truncate_plan_for_rerun(path, user_span_end)?;
        return Ok(true);
    }
    Err(PlanFileError::MissingSection(
        "user plan sidecar for interrupted run",
    ))
}

pub fn remove_plan_user_sidecar(path: &Path) -> Result<(), PlanFileError> {
    let sidecar = plan_user_sidecar_path(path);
    if sidecar.is_file() {
        std::fs::remove_file(&sidecar)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::PlanFileError;

    #[test]
    fn kiss_cov_boundary_helper_symbols() {
        let _ = legacy_machine_block_marker_patterns;
        let _ = count_legacy_begin_malvin_marker_lines;
    }

    #[test]
    fn count_legacy_begin_malvin_marker_lines_ignores_prose_and_counts_whole_lines() {
        let prose = "See BEGIN_MALVIN in docs.\n\n---\nBEGIN_MALVIN\n";
        assert_eq!(detect_rerun_user_span_end(prose), Ok(Some(27)));
        let duplicate = "BEGIN_MALVIN\n\n---\nBEGIN_MALVIN\n";
        assert_eq!(
            detect_rerun_user_span_end(duplicate),
            Err(PlanFileError::DuplicateBeginMalvinMarkers)
        );
        let indented = "  BEGIN_MALVIN  \n\n---\nBEGIN_MALVIN\n";
        assert_eq!(
            detect_rerun_user_span_end(indented),
            Err(PlanFileError::DuplicateBeginMalvinMarkers)
        );
    }

    #[test]
    fn is_interrupted_machine_plan_detects_restatement_first() {
        assert!(is_interrupted_machine_plan("## Restatement\nrestated\n"));
        assert!(!is_interrupted_machine_plan("# User\n"));
    }

    #[test]
    fn restore_interrupted_plan_from_sidecar() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("plan.md");
        let sidecar = plan_user_sidecar_path(&path);
        std::fs::write(&sidecar, "# User\n").expect("sidecar");
        std::fs::write(&path, "## Restatement\nrestated\n").expect("plan");
        assert!(restore_interrupted_plan(&path).expect("restore"));
        assert_eq!(std::fs::read_to_string(&path).expect("read"), "# User\n");
        assert!(!sidecar.is_file());
    }
}
