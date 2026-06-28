//! Workspace layout paths (`.malvin/` tree) and home-directory run logs.

use std::path::{Path, PathBuf};

#[path = "workspace_paths_data_root.rs"]
pub(crate) mod workspace_paths_data_root;

pub use workspace_paths_data_root::{
    git_worktree_toplevel, legacy_malvin_checks_path, malvin_acp_spawn_chamber_dir,
    malvin_checks_path, malvin_data_root, resolve_malvin_checks_path,
};

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

/// When set to `1` during `cargo test`, code may create/write/delete `~/.malvin_home/config.toml`.
pub const MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION: &str = "MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION";

/// Whether home-config disk mutation may run (always true outside test builds).
pub(crate) fn home_malvin_config_disk_io_allowed() -> bool {
    if cfg!(test) {
        std::env::var(MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION).as_deref() == Ok("1")
    } else {
        true
    }
}

/// Refuse home-config disk mutation in test builds when isolation consent is absent.
pub(crate) fn assert_home_malvin_config_disk_io_allowed(op: &str) -> Result<(), String> {
    if home_malvin_config_disk_io_allowed() {
        Ok(())
    } else {
        Err(format!(
            "refusing {op} on ~/.malvin_home/config.toml without test isolation; \
             use with_isolated_home or activate_test_home (see plan.md)"
        ))
    }
}

/// Whether home-config delete/recreate paths may run (always true outside test builds).
pub(crate) fn home_malvin_config_delete_allowed() -> bool {
    home_malvin_config_disk_io_allowed()
}

/// Run-directory file recording the canonical workspace cwd for this run.
pub const WORK_DIR_MANIFEST: &str = "work_dir";

const LEGACY_MALVIN_CHECKS_FILE: &str = ".malvin_checks";

const FNV1A_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV1A_PRIME: u64 = 0x0000_0100_0000_01B3;

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

/// `~/.malvin/snapshots/` — session dotfile backups (kissconfig, gitignore tree, etc.).
pub const MALVIN_SNAPSHOTS_DIR: &str = "snapshots";

#[must_use]
pub fn malvin_home_snapshots_root() -> PathBuf {
    crate::user_home_dir()
        .join(".malvin")
        .join(MALVIN_SNAPSHOTS_DIR)
}

#[must_use]
pub fn snapshot_category_dir(category: &str) -> PathBuf {
    malvin_home_snapshots_root().join(category)
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

