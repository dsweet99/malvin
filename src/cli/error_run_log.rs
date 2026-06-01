//! Optional `.malvin/logs/.../malvin_error.log` mirror for fatal CLI errors (see [`append_command_error_to_run_log`]).
//!
//! The active run directory is set by each subcommand once [`RunArtifacts`] exists so
//! [`super::entrypoint::print_command_error`] can append even when the message only went to stderr before.

use std::path::PathBuf;
use std::sync::Mutex;

use crate::output::{ERROR_WHO, format_line};

static COMMAND_ERROR_RUN_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Remember `.malvin/logs/<stamp>/` for [`append_command_error_to_run_log`].
pub fn set_command_error_run_dir(path: Option<PathBuf>) {
    *COMMAND_ERROR_RUN_DIR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = path;
}

/// Clears the directory installed by [`set_command_error_run_dir`].
pub fn clear_command_error_run_dir() {
    set_command_error_run_dir(None);
}

/// Appends one timestamped malvin line to `malvin_error.log` under the bound run directory, when set.
pub fn append_command_error_to_run_log(message: &str) {
    if crate::repo_checks::is_gate_failure_error(message) {
        return;
    }
    let Some(dir) = COMMAND_ERROR_RUN_DIR
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
    else {
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
    use crate::output::{format_who_tag_delim, ERROR_WHO};
    use tempfile::tempdir;

    #[test]
    fn append_command_error_writes_malvin_error_log() {
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
}
