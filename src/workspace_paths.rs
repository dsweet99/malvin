//! Workspace layout paths (`.malvin/` tree).

use std::path::{Path, PathBuf};

pub const MALVIN_DIR: &str = ".malvin";

pub const MALVIN_CHECKS_REL: &str = ".malvin/checks";

pub const MALVIN_ADVICE_REL: &str = ".malvin/advice.md";

pub const MALVIN_LOGS_REL: &str = ".malvin/logs";

const LEGACY_MALVIN_CHECKS_FILE: &str = ".malvin_checks";

#[must_use]
pub fn malvin_checks_path(work_dir: &Path) -> PathBuf {
    work_dir.join(MALVIN_CHECKS_REL)
}

#[must_use]
pub fn malvin_advice_path(work_dir: &Path) -> PathBuf {
    work_dir.join(MALVIN_ADVICE_REL)
}

#[must_use]
pub fn malvin_logs_root(work_dir: &Path) -> PathBuf {
    work_dir.join(MALVIN_LOGS_REL)
}

/// Walks from `start` toward the filesystem root until a `.malvin/logs` directory exists.
#[must_use]
pub fn find_malvin_logs_root(mut start: &Path) -> Option<PathBuf> {
    loop {
        let candidate = malvin_logs_root(start);
        if candidate.is_dir() {
            return Some(candidate);
        }
        start = start.parent()?;
        if start.as_os_str().is_empty() {
            return None;
        }
    }
}

#[must_use]
pub fn is_malvin_workspace(work_dir: &Path) -> bool {
    work_dir.join(MALVIN_DIR).is_dir()
}

/// Removes pre-migration root `.malvin_checks` if present (no legacy fallback reads).
pub fn remove_legacy_malvin_checks_file(work_dir: &Path) {
    let legacy = work_dir.join(LEGACY_MALVIN_CHECKS_FILE);
    if legacy.is_file() {
        let _ = std::fs::remove_file(legacy);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_helpers_and_workspace_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        assert_eq!(malvin_checks_path(w), w.join(MALVIN_CHECKS_REL));
        assert_eq!(malvin_advice_path(w), w.join(MALVIN_ADVICE_REL));
        assert_eq!(malvin_logs_root(w), w.join(MALVIN_LOGS_REL));
        assert!(!is_malvin_workspace(w));
        std::fs::create_dir_all(w.join(MALVIN_DIR)).unwrap();
        assert!(is_malvin_workspace(w));
    }

    #[test]
    fn find_malvin_logs_root_walks_up_from_subdirectory() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        let sub = w.join("pkg").join("src");
        std::fs::create_dir_all(&sub).unwrap();
        let logs = malvin_logs_root(w);
        std::fs::create_dir_all(&logs).unwrap();
        assert_eq!(find_malvin_logs_root(&sub).as_deref(), Some(logs.as_path()));
    }

    #[test]
    fn remove_legacy_malvin_checks_file_deletes_legacy_not_layout_checks() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path();
        std::fs::write(w.join(LEGACY_MALVIN_CHECKS_FILE), "legacy\n").unwrap();
        std::fs::create_dir_all(w.join(MALVIN_DIR)).unwrap();
        std::fs::write(malvin_checks_path(w), "current\n").unwrap();
        remove_legacy_malvin_checks_file(w);
        assert!(!w.join(LEGACY_MALVIN_CHECKS_FILE).exists());
        assert_eq!(std::fs::read_to_string(malvin_checks_path(w)).unwrap(), "current\n");
    }
}
