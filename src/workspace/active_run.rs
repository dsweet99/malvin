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

#[must_use]
pub fn active_run_dir() -> Option<PathBuf> {
    ACTIVE_RUN_DIR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

#[must_use]
pub(crate) fn short_malvin_log_id(run_dir: &Path) -> Option<String> {
    let run = run_dir.file_name()?.to_str()?;
    let hash = run_dir.parent()?.file_name()?.to_str()?;
    if run.is_empty() || hash.is_empty() {
        return None;
    }
    Some(format!("{hash}/{run}"))
}
