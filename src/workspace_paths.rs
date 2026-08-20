use std::path::{Path, PathBuf};

#[path = "workspace_paths_data_root.rs"]
pub(crate) mod workspace_paths_data_root;

pub use workspace_paths_data_root::{
    git_worktree_toplevel, legacy_malvin_checks_path, malvin_acp_spawn_chamber_dir,
    malvin_checks_path, malvin_data_root, resolve_malvin_checks_path,
};

pub const MALVIN_DIR: &str = ".malvin";

pub const MALVIN_CHECKS_REL: &str = ".malvin/gates";

pub const MALVIN_CHECKS_LEGACY_REL: &str = ".malvin/checks";

pub const MALVIN_ADVICE_REL: &str = ".malvin/advice.md";

pub const MALVIN_LOGS_REL: &str = ".malvin/logs";

pub const MALVIN_CONFIG_REL: &str = ".malvin/config.toml";

pub const MALVIN_USER_HOME_DIR: &str = ".malvin_home";

pub const MALVIN_HOME_CONFIG_FILE: &str = "config.toml";

pub const MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION: &str = "MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION";

pub(crate) fn home_malvin_config_disk_io_allowed() -> bool {
    if cfg!(test) {
        std::env::var(MALVIN_TEST_ALLOW_HOME_CONFIG_MUTATION).as_deref() == Ok("1")
    } else {
        true
    }
}

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

pub(crate) fn home_malvin_config_delete_allowed() -> bool {
    home_malvin_config_disk_io_allowed()
}

pub const WORK_DIR_MANIFEST: &str = "work_dir";

const LEGACY_MALVIN_CHECKS_FILE: &str = ".malvin_checks";

const FNV1A_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV1A_PRIME: u64 = 0x0000_0100_0000_01B3;

#[must_use]
pub fn malvin_advice_path(work_dir: &Path) -> PathBuf {
    work_dir.join(MALVIN_ADVICE_REL)
}

#[must_use]
pub fn malvin_logs_root(work_dir: &Path) -> PathBuf {
    malvin_home_logs_root().join(workspace_logs_hash(work_dir))
}

#[must_use]
pub fn malvin_user_home_root() -> PathBuf {
    let configured = test_storage_home_dir().join(MALVIN_USER_HOME_DIR);
    #[cfg(test)]
    {
        if directory_is_writable_for_tests(&configured) {
            return configured;
        }
        std::env::temp_dir()
            .join(format!("malvin-test-home-{}", std::process::id()))
            .join(MALVIN_USER_HOME_DIR)
    }
    #[cfg(not(test))]
    configured
}

#[cfg(test)]
fn directory_is_writable_for_tests(path: &Path) -> bool {
    if std::fs::create_dir_all(path).is_err() {
        return false;
    }
    let probe = path.join(format!(".write-probe-{}", std::process::id()));
    let writable = std::fs::write(&probe, []).is_ok();
    let _ = std::fs::remove_file(probe);
    writable
}

#[must_use]
pub fn malvin_home_logs_root() -> PathBuf {
    malvin_user_home_root().join("logs")
}

pub const MALVIN_SNAPSHOTS_DIR: &str = "snapshots";

#[must_use]
pub fn malvin_home_snapshots_root() -> PathBuf {
    test_storage_home_dir()
        .join(".malvin")
        .join(MALVIN_SNAPSHOTS_DIR)
}

fn test_storage_home_dir() -> PathBuf {
    #[cfg(test)]
    {
        let configured = crate::user_home_dir();
        let probe = configured.join(format!(".malvin-write-probe-{}", std::process::id()));
        if std::fs::write(&probe, []).is_ok() {
            let _ = std::fs::remove_file(probe);
            return configured;
        }
        std::env::temp_dir().join(format!("malvin-test-home-{}", std::process::id()))
    }
    #[cfg(not(test))]
    crate::user_home_dir()
}

#[must_use]
pub fn snapshot_category_dir(category: &str) -> PathBuf {
    malvin_home_snapshots_root().join(category)
}

#[must_use]
pub fn malvin_config_path(_work_dir: &Path) -> PathBuf {
    malvin_home_config_path()
}

#[must_use]
pub fn malvin_home_config_path() -> PathBuf {
    malvin_user_home_root().join(MALVIN_HOME_CONFIG_FILE)
}

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
        std::env::current_dir().map_or_else(|_| work_dir.to_path_buf(), |cwd| cwd.join(work_dir))
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

pub fn write_work_dir_manifest(run_dir: &Path, work_dir: &Path) -> std::io::Result<()> {
    let abs = canonical_work_dir_for_logs(work_dir);
    std::fs::write(
        run_dir.join(WORK_DIR_MANIFEST),
        format!("{}\n", abs.display()),
    )
}

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

pub fn remove_legacy_malvin_checks_file(work_dir: &Path) {
    let legacy = work_dir.join(LEGACY_MALVIN_CHECKS_FILE);
    if legacy.is_file() {
        let _ = std::fs::remove_file(legacy);
    }
}
