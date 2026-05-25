//! Run directories and log paths.

mod md_request;
mod startup_tag;
mod create;

use std::path::{Path, PathBuf};

pub use create::{
    create_kpop_run_artifacts, create_kpop_run_artifacts_opts, create_run_artifacts,
    create_run_artifacts_from_text, create_run_artifacts_from_text_opts, create_run_artifacts_opts,
};

pub use crate::session_dotfile_backup::{
    KissConfigBackup, KissignoreBackup, MalvinChecksBackup, SessionDotfileBackups,
    backup_workspace_kissconfig_if_present, backup_workspace_kissconfig_if_present_with_id,
    backup_workspace_kissignore_if_present, backup_workspace_kissignore_if_present_with_id,
    backup_workspace_malvin_checks_if_present, backup_workspace_malvin_checks_if_present_with_id,
    restore_workspace_kissconfig_backup, restore_workspace_kissignore_backup,
    restore_workspace_malvin_checks_backup, restore_workspace_session_dotfiles,
};

pub use md_request::{is_existing_md_file_path, resolve_user_md_request};
pub use startup_tag::startup_request_tag_label;

pub use crate::malvin_constants::{QUALITY_GATES_LOG, STDOUT_LOG, TRACE_JSONL};

/// One workflow run: isolated `.malvin/logs/<stamp>_<token>/` with copied plan.
#[derive(Debug, Clone)]
pub struct RunArtifacts {
    pub run_dir: PathBuf,
    pub plan_path: PathBuf,
    pub work_dir: PathBuf,
}

impl RunArtifacts {
    #[must_use]
    pub fn log_path(&self, name: &str) -> PathBuf {
        let safe = name.replace(['/', '\\'], "_");
        self.run_dir.join(format!("{safe}.log"))
    }

    /// Run-directory copy of `review.md` (artifact for [`crate::review_sync`]).
    #[must_use]
    pub fn artifact_review_md(&self) -> PathBuf {
        self.run_dir.join("review.md")
    }

    #[must_use]
    pub fn review_prep_md(&self) -> PathBuf {
        self.run_dir.join("review_prep.md")
    }

    /// Workspace `review.md` under [`Self::work_dir`].
    #[must_use]
    pub fn workspace_review_md(&self) -> PathBuf {
        self.work_dir.join("review.md")
    }

    /// Run-directory `result.md` for concerns ABORT signaling.
    #[must_use]
    pub fn artifact_result_md(&self) -> PathBuf {
        self.run_dir.join("result.md")
    }

    #[must_use]
    pub fn exp_log_path(&self) -> PathBuf {
        let slug = self
            .run_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("run");
        self.run_dir
            .join("_kpop")
            .join(format!("exp_log_{slug}.md"))
    }

    #[must_use]
    pub fn quality_gates_log_path(&self) -> PathBuf {
        self.run_dir.join(QUALITY_GATES_LOG)
    }

    #[must_use]
    pub fn stdout_log_path(&self) -> PathBuf {
        self.run_dir.join(STDOUT_LOG)
    }

    #[must_use]
    pub fn trace_jsonl_path(&self) -> PathBuf {
        self.run_dir.join(TRACE_JSONL)
    }
}

pub(crate) fn work_dir_for_path(path: &Path) -> PathBuf {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(
            || PathBuf::from("."),
            |parent| parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf()),
        )
}

pub(crate) fn resolve_user_at_path(rest: &str) -> Result<PathBuf, String> {
    if rest.is_empty() {
        return Err("Empty path after `@`.".to_string());
    }
    let path = Path::new(rest);
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let resolved = if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    };
    if resolved.is_file() {
        return Ok(resolved);
    }
    if resolved.is_dir() {
        return Err(format!(
            "Path is a directory, not a file: {}",
            resolved.display()
        ));
    }
    Err(format!("Path is not a file: {}", resolved.display()))
}

pub(crate) fn resolve_at_file(rest: &str) -> Result<(String, PathBuf), String> {
    let path = resolve_user_at_path(rest)?;
    let text = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    Ok((text, work_dir_for_path(&path)))
}

/// Resolve CLI `request`: `@path` reads an existing file; otherwise treat as literal text.
///
/// # Errors
///
/// Returns a message when `@` is used but the path is missing or unreadable.
pub fn resolve_user_request(arg: &str) -> Result<(String, PathBuf), String> {
    let arg = arg.trim();
    arg.strip_prefix('@').map_or_else(
        || Ok((arg.to_string(), PathBuf::from("."))),
        resolve_at_file,
    )
}

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "kpop_path_tests.rs"]
mod kpop_path_tests;
