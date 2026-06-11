//! Workspace layout paths (`.malvin/` tree) and home-directory run logs.

use std::path::{Path, PathBuf};

pub const MALVIN_DIR: &str = ".malvin";

pub const MALVIN_CHECKS_REL: &str = ".malvin/checks";

pub const MALVIN_ADVICE_REL: &str = ".malvin/advice.md";

/// Legacy repo-relative logs path (pre-relocation); retained for docs and migration tests.
pub const MALVIN_LOGS_REL: &str = ".malvin/logs";

/// Workspace-local config (distinct from [`malvin_home_config_path`]).
pub const MALVIN_CONFIG_REL: &str = ".malvin/config.toml";

/// Per-user malvin data root under `$HOME` (logs, config, names, dotfile snapshots).
pub const MALVIN_USER_HOME_DIR: &str = ".malvin_home";

/// Global user config filename under [`malvin_user_home_root`].
pub const MALVIN_HOME_CONFIG_FILE: &str = "config.toml";

/// Run-directory file recording the canonical workspace cwd for this run.
pub const WORK_DIR_MANIFEST: &str = "work_dir";

const LEGACY_MALVIN_CHECKS_FILE: &str = ".malvin_checks";

const FNV1A_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV1A_PRIME: u64 = 0x0000_0100_0000_01B3;

#[must_use]
pub fn malvin_checks_path(work_dir: &Path) -> PathBuf {
    work_dir.join(MALVIN_CHECKS_REL)
}

#[must_use]
pub fn malvin_advice_path(work_dir: &Path) -> PathBuf {
    work_dir.join(MALVIN_ADVICE_REL)
}

/// `~/.malvin_home/logs/<hash>/` for the canonical absolute path of `work_dir`.
#[must_use]
pub fn malvin_logs_root(work_dir: &Path) -> PathBuf {
    malvin_home_logs_root().join(workspace_logs_hash(work_dir))
}

/// `~/.malvin_home/`.
#[must_use]
pub fn malvin_user_home_root() -> PathBuf {
    crate::user_home_dir().join(MALVIN_USER_HOME_DIR)
}

#[must_use]
pub fn malvin_home_logs_root() -> PathBuf {
    malvin_user_home_root().join("logs")
}

/// `~/.malvin_home/config.toml` (global user config; `work_dir` is ignored).
#[must_use]
pub fn malvin_config_path(_work_dir: &Path) -> PathBuf {
    malvin_home_config_path()
}

#[must_use]
pub fn malvin_home_config_path() -> PathBuf {
    malvin_user_home_root().join(MALVIN_HOME_CONFIG_FILE)
}

/// Alphanumeric (hex) digest of the canonical absolute path of `work_dir`.
#[must_use]
pub fn workspace_logs_hash(work_dir: &Path) -> String {
    let abs = canonical_work_dir_for_logs(work_dir);
    format!("{:016x}", fnv1a64(abs.as_os_str().as_encoded_bytes()))
}

#[must_use]
pub fn canonical_work_dir_for_logs(work_dir: &Path) -> PathBuf {
    let resolved = if work_dir.is_absolute() {
        work_dir.to_path_buf()
    } else {
        std::env::current_dir().map_or_else(
            |_| work_dir.to_path_buf(),
            |cwd| cwd.join(work_dir),
        )
    };
    resolved.canonicalize().unwrap_or(resolved)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = FNV1A_OFFSET;
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(FNV1A_PRIME);
    }
    hash
}

/// Returns the home logs bucket for `start` when it already exists.
#[must_use]
pub fn find_malvin_logs_root(start: &Path) -> Option<PathBuf> {
    let candidate = malvin_logs_root(start);
    if candidate.is_dir() {
        Some(candidate)
    } else {
        None
    }
}

#[must_use]
pub fn is_malvin_workspace(work_dir: &Path) -> bool {
    work_dir.join(MALVIN_DIR).is_dir()
}

/// Writes the canonical workspace path into a run directory manifest.
///
/// # Errors
///
/// Returns [`std::io::Error`] when the manifest cannot be written.
pub fn write_work_dir_manifest(run_dir: &Path, work_dir: &Path) -> std::io::Result<()> {
    let abs = canonical_work_dir_for_logs(work_dir);
    std::fs::write(run_dir.join(WORK_DIR_MANIFEST), format!("{}\n", abs.display()))
}

/// Reads the workspace cwd recorded for a run, if present.
#[must_use]
pub fn read_work_dir_manifest(run_dir: &Path) -> Option<PathBuf> {
    let path = run_dir.join(WORK_DIR_MANIFEST);
    let text = std::fs::read_to_string(&path).ok()?;
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(PathBuf::from(trimmed))
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
        assert_eq!(malvin_config_path(w), malvin_home_config_path());
        assert!(!is_malvin_workspace(w));
        std::fs::create_dir_all(w.join(MALVIN_DIR)).unwrap();
        assert!(is_malvin_workspace(w));
    }

    #[test]
    fn workspace_logs_hash_is_stable_hex() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path().join("proj");
        std::fs::create_dir_all(&w).unwrap();
        let h1 = workspace_logs_hash(&w);
        let h2 = workspace_logs_hash(&w);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 16);
        assert!(h1.bytes().all(|b| b.is_ascii_hexdigit()));
    }

    #[test]
    fn workspace_logs_hash_differs_for_different_paths() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();
        assert_ne!(workspace_logs_hash(&a), workspace_logs_hash(&b));
    }

    #[test]
    fn malvin_user_home_root_uses_malvin_home_dir() {
        let root = malvin_user_home_root();
        assert!(root.ends_with(MALVIN_USER_HOME_DIR));
        assert!(root.starts_with(crate::user_home_dir()));
    }

    #[test]
    fn malvin_logs_root_lives_under_home_not_workspace() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path().join("ws");
        std::fs::create_dir_all(&w).unwrap();
        let root = malvin_logs_root(&w);
        assert!(root.starts_with(malvin_home_logs_root()));
        assert!(!root.starts_with(w));
    }

    #[test]
    fn find_malvin_logs_root_none_until_bucket_exists() {
        let tmp = tempfile::tempdir().unwrap();
        let w = tmp.path().join("fresh");
        std::fs::create_dir_all(&w).unwrap();
        assert_eq!(find_malvin_logs_root(&w), None);
        let bucket = malvin_logs_root(&w);
        std::fs::create_dir_all(&bucket).unwrap();
        assert_eq!(find_malvin_logs_root(&w).as_deref(), Some(bucket.as_path()));
    }

    #[test]
    fn work_dir_manifest_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let ws = tmp.path().join("ws");
        let run = tmp.path().join("run");
        std::fs::create_dir_all(&ws).unwrap();
        std::fs::create_dir_all(&run).unwrap();
        write_work_dir_manifest(&run, &ws).unwrap();
        let read = read_work_dir_manifest(&run).expect("manifest");
        assert_eq!(read, canonical_work_dir_for_logs(&ws));
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
