use chrono::Utc;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static ACTIVE_RUN_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

#[cfg(test)]
pub(crate) static ACTIVE_RUN_DIR_TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn set_active_run_dir(path: Option<PathBuf>) {
    *ACTIVE_RUN_DIR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = path;
}

/// Bind the active run directory and notify herdr that a run started.
/// Owns run-lifecycle side effects that used to live behind `error_run_log`.
pub(crate) fn activate_run(path: PathBuf) {
    set_active_run_dir(Some(path.clone()));
    crate::herdr::notify_run_start(&path);
}

/// Notify herdr that the run ended and clear the active run directory.
pub(crate) fn deactivate_run() {
    crate::herdr::notify_run_end();
    set_active_run_dir(None);
}

#[must_use]
pub(crate) fn active_run_dir() -> Option<PathBuf> {
    ACTIVE_RUN_DIR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

/// Unique log path under `~/.malvin_home/logs/`: `<hash>/<run_id>`.
#[must_use]
pub(crate) fn short_malvin_log_id(run_dir: &Path) -> Option<String> {
    let run = run_dir.file_name()?.to_str()?;
    let hash = run_dir.parent()?.file_name()?.to_str()?;
    if run.is_empty() || hash.is_empty() {
        return None;
    }
    Some(format!("{hash}/{run}"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunDirOptions {
    pub gc: bool,
}

impl Default for RunDirOptions {
    fn default() -> Self {
        Self {
            gc: default_gc_enabled(),
        }
    }
}

fn default_gc_enabled() -> bool {
    if cfg!(test) {
        return false;
    }
    let Ok(exe) = std::env::current_exe() else {
        return true;
    };
    let path = exe.to_string_lossy();
    !(path.contains("/deps/") || path.contains("\\deps\\"))
}

pub fn create_run_dir(base_dir: Option<&Path>, opts: RunDirOptions) -> std::io::Result<PathBuf> {
    let parent = base_dir.unwrap_or_else(|| Path::new("."));
    let run_root = crate::malvin_logs_root(parent);
    std::fs::create_dir_all(&run_root)?;
    let run_dir = create_run_dir_with_id(&run_root, |_| build_identifier())?;
    if opts.gc {
        gc_after_run_created(parent, &run_dir);
    }
    Ok(run_dir)
}

fn gc_after_run_created(base_dir: &Path, run_dir: &Path) {
    crate::log_gc::prune_logs_after_run_created(base_dir, run_dir);
    if crate::malvin_acp_spawn_chamber_dir(base_dir).is_dir() {
        let _ = crate::acp_spawn_sweep::sweep_stale_acp_spawn_locks(base_dir);
    }
}

pub fn maybe_gc_after_run_created(base_dir: &Path, run_dir: &Path) {
    if !default_gc_enabled() {
        return;
    }
    gc_after_run_created(base_dir, run_dir);
}

#[must_use]
pub fn build_identifier() -> String {
    let stamp = Utc::now().format("%Y%m%d_%H%M%S");
    let token = random_alnum(8);
    format!("{stamp}_{token}")
}

pub use crate::alnum_id::random_alnum;

fn create_run_dir_with_id(
    run_root: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> std::io::Result<PathBuf> {
    let mut tries = 0usize;
    std::fs::create_dir_all(run_root)?;
    while tries < 16 {
        let identifier = generate_id(tries);
        let run_dir = run_root.join(&identifier);
        match std::fs::create_dir(&run_dir) {
            Ok(()) => return Ok(run_dir),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                tries += 1;
            }
            Err(err) => return Err(err),
        }
    }
    Err(std::io::Error::new(
        ErrorKind::AlreadyExists,
        "run directory id collision limit exceeded",
    ))
}

#[cfg(test)]
mod short_log_id_tests {
    use super::*;

    #[test]
    fn short_malvin_log_id_takes_hash_and_run() {
        let path = Path::new("/home/dsweet/.malvin_home/logs/eb7ef333a92a6d41/20260830_024330_estp91hf");
        assert_eq!(
            short_malvin_log_id(path).as_deref(),
            Some("eb7ef333a92a6d41/20260830_024330_estp91hf")
        );
    }

    #[test]
    fn short_malvin_log_id_none_without_parent() {
        assert!(short_malvin_log_id(Path::new("only_run")).is_none());
        assert!(short_malvin_log_id(Path::new("/")).is_none());
    }

    #[test]
    fn active_run_dir_round_trip() {
        let _guard = ACTIVE_RUN_DIR_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        set_active_run_dir(None);
        assert!(active_run_dir().is_none());
        let path = PathBuf::from("/tmp/malvin-active-run/hash/run");
        set_active_run_dir(Some(path.clone()));
        assert_eq!(active_run_dir(), Some(path));
        set_active_run_dir(None);
        assert!(active_run_dir().is_none());
    }
}

#[cfg(test)]
mod collision_tests {
    use super::*;

    #[test]
    fn create_run_dir_retries_collision_ids() {
        let tmp = tempfile::tempdir().unwrap();
        let run_root = crate::malvin_logs_root(tmp.path());
        std::fs::create_dir_all(&run_root).unwrap();
        std::fs::create_dir_all(run_root.join("aaabbbcc")).unwrap();

        let run_dir = create_run_dir_with_id(&run_root, |attempt| {
            if attempt == 0 {
                "aaabbbcc".to_string()
            } else {
                "aaabbbcd".to_string()
            }
        })
        .unwrap();

        assert_eq!(run_dir, run_root.join("aaabbbcd"));
        assert!(run_dir.is_dir());
    }

    #[test]
    fn create_run_dir_and_build_identifier_smoke() {
        let tmp = tempfile::tempdir().unwrap();
        let id = build_identifier();
        assert!(!id.is_empty());
        let dir = create_run_dir(Some(tmp.path()), RunDirOptions::default()).unwrap();
        assert!(dir.is_dir());
    }

    #[test]
    fn default_run_dir_options_disable_gc_under_cfg_test() {
        assert!(
            !RunDirOptions::default().gc,
            "lib unit tests must not walk the real home logs tree by default"
        );
        assert!(!default_gc_enabled());
    }
}
