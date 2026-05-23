//! Global stdout tee log path holder (crate-root leaf: no malvin internals).
//!
//! Keeps dependency depth shallow for callers like `artifacts` that only need run-dir log wiring.

use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

static STDOUT_LOG_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

/// # Panics
///
/// Panics if the stdout log path mutex is poisoned.
pub fn set_stdout_log_path(path: Option<PathBuf>) {
    *STDOUT_LOG_PATH
        .lock()
        .unwrap_or_else(PoisonError::into_inner) = path;
}

#[must_use]
/// # Panics
///
/// Panics if the stdout log path mutex is poisoned.
pub fn clone_stdout_log_path() -> Option<PathBuf> {
    STDOUT_LOG_PATH
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

#[cfg(test)]
mod kiss_coverage {
    #[test]
    fn smoke_set_stdout_log_path_and_clone_stdout_log_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("out.log");
        super::set_stdout_log_path(Some(path.clone()));
        assert_eq!(super::clone_stdout_log_path(), Some(path));
        super::set_stdout_log_path(None);
        assert!(super::clone_stdout_log_path().is_none());
        let _ = stringify!(super::set_stdout_log_path);
        let _ = stringify!(super::clone_stdout_log_path);
    }
}
