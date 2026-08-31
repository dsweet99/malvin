use std::path::PathBuf;
use std::sync::Mutex;

use crate::output::{ERROR_WHO, format_line};

static LAST_EMITTED_COMMAND_ERROR: Mutex<Option<String>> = Mutex::new(None);

pub fn set_command_error_run_dir(path: Option<PathBuf>) {
    crate::run_id::set_active_run_dir(path.clone());
    if let Some(run_dir) = path.as_ref() {
        crate::herdr::notify_run_start(run_dir);
    }
}

#[cfg(test)]
pub fn command_error_run_dir() -> Option<PathBuf> {
    crate::run_id::active_run_dir()
}

pub fn clear_command_error_run_dir() {
    crate::herdr::notify_run_end();
    set_command_error_run_dir(None);
    *LAST_EMITTED_COMMAND_ERROR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

#[must_use]
pub fn command_error_already_emitted(message: &str) -> bool {
    LAST_EMITTED_COMMAND_ERROR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_deref()
        == Some(message)
}

pub fn note_command_error_emitted(message: &str) {
    *LAST_EMITTED_COMMAND_ERROR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(message.to_string());
}

pub fn append_command_error_to_run_log(message: &str) {
    if crate::repo_checks::is_gate_failure_error(message) {
        return;
    }
    let Some(dir) = crate::run_id::active_run_dir() else {
        return;
    };
    let path = dir.join("malvin_error.log");
    let line = format!("{}\n", format_line(ERROR_WHO, message));
    let _ = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, line.as_bytes()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{ERROR_WHO, format_who_tag_delim};
    use tempfile::tempdir;

    #[test]
    fn command_error_run_dir_reads_active_binding() {
        let _lock = crate::run_id::ACTIVE_RUN_DIR_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        clear_command_error_run_dir();
        assert_eq!(command_error_run_dir(), None);
        let dir = tempdir().expect("tempdir");
        let path = dir.path().to_path_buf();
        set_command_error_run_dir(Some(path.clone()));
        assert_eq!(command_error_run_dir(), Some(path));
        clear_command_error_run_dir();
        assert_eq!(command_error_run_dir(), None);
    }

    #[test]
    fn append_command_error_writes_malvin_error_log() {
        let _lock = crate::run_id::ACTIVE_RUN_DIR_TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let dir = tempdir().expect("tempdir");
        set_command_error_run_dir(Some(dir.path().to_path_buf()));
        append_command_error_to_run_log("something went wrong");
        clear_command_error_run_dir();
        let text = std::fs::read_to_string(dir.path().join("malvin_error.log")).expect("read log");
        assert!(
            text.contains("something went wrong"),
            "unexpected log contents: {text:?}"
        );
        assert!(
            text.contains(&format_who_tag_delim(ERROR_WHO)),
            "expected error tag in log line: {text:?}"
        );
    }

    #[test]
    fn command_error_emit_dedupes_identical_message() {
        clear_command_error_run_dir();
        assert!(!command_error_already_emitted("dup"));
        note_command_error_emitted("dup");
        assert!(command_error_already_emitted("dup"));
        assert!(!command_error_already_emitted("other"));
        clear_command_error_run_dir();
        assert!(!command_error_already_emitted("dup"));
    }
}
