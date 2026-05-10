//! Run directories and log paths.

mod dotfile_backup;
mod kiss_config_backup;
mod malvin_checks_backup;
pub mod run_id;
mod startup_tag;

pub use kiss_config_backup::{
    KissConfigBackup, backup_workspace_kissconfig_if_present, restore_workspace_kissconfig_backup,
};
pub use malvin_checks_backup::{
    MalvinChecksBackup, backup_workspace_malvin_checks_if_present,
    restore_workspace_malvin_checks_backup,
};

use std::path::{Path, PathBuf};

pub use startup_tag::startup_request_tag_label;

/// One workflow run: isolated `_malvin/<stamp>_<token>/` with copied plan.
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
}

/// Copy `plan_source` into a fresh run directory under `base_dir`/`_malvin`/…
///
/// # Errors
///
/// Returns an I/O error if directories cannot be created or the plan cannot be copied.
pub fn create_run_artifacts(
    plan_source: &Path,
    base_dir: Option<&Path>,
) -> std::io::Result<RunArtifacts> {
    let run_dir = run_id::create_run_dir(base_dir)?;
    let plan_target = run_dir.join("plan.md");
    std::fs::copy(plan_source, &plan_target)?;
    Ok(RunArtifacts {
        run_dir,
        plan_path: plan_target,
        work_dir: plan_source
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf),
    })
}

/// Write `plan_text` into a fresh run directory under `base_dir`/`_malvin`/…
///
/// # Errors
///
/// Returns an I/O error if directories cannot be created or the plan text cannot be written.
pub fn create_run_artifacts_from_text(
    plan_text: &str,
    base_dir: Option<&Path>,
) -> std::io::Result<RunArtifacts> {
    let work_dir = base_dir.unwrap_or_else(|| Path::new(".")).to_path_buf();
    let run_dir = run_id::create_run_dir(base_dir)?;
    let plan_target = run_dir.join("plan.md");
    std::fs::write(&plan_target, plan_text)?;
    Ok(RunArtifacts {
        run_dir,
        plan_path: plan_target,
        work_dir,
    })
}

/// Write `request_text` to `_malvin/.../request.md` for standalone `kpop` runs.
///
/// [`RunArtifacts::plan_path`] points at `request.md` so templates can resolve a stable path.
///
/// # Errors
///
/// Returns an I/O error if directories cannot be created or the request text cannot be written.
pub fn create_kpop_run_artifacts(
    request_text: &str,
    base_dir: Option<&Path>,
) -> std::io::Result<RunArtifacts> {
    let work_dir = base_dir.unwrap_or_else(|| Path::new(".")).to_path_buf();
    let run_dir = run_id::create_run_dir(base_dir)?;
    let request_target = run_dir.join("request.md");
    std::fs::write(&request_target, request_text)?;
    Ok(RunArtifacts {
        run_dir,
        plan_path: request_target,
        work_dir,
    })
}

pub(crate) fn work_dir_for_path(path: &Path) -> PathBuf {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

pub(crate) fn resolve_at_file(rest: &str) -> Result<(String, PathBuf), String> {
    if rest.is_empty() {
        return Err("Empty path after `@`.".to_string());
    }
    let path = Path::new(rest);
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok((text, work_dir_for_path(path)))
}

/// Restore workspace `.kissconfig` and `.malvin_checks` from session backups (see [`backup_workspace_kissconfig_if_present`], [`backup_workspace_malvin_checks_if_present`]).
///
/// Both restores are attempted; if both fail, errors are joined with `; `.
///
/// # Errors
///
/// Returns [`Err`] when either underlying restore fails; see those functions for failure cases.
pub fn restore_workspace_session_dotfiles(
    work_dir: &Path,
    kissconfig_backup: &KissConfigBackup,
    malvin_checks_backup: &MalvinChecksBackup,
) -> Result<(), String> {
    let k = restore_workspace_kissconfig_backup(work_dir, kissconfig_backup);
    let m = restore_workspace_malvin_checks_backup(work_dir, malvin_checks_backup);
    match (k, m) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Ok(())) | (Ok(()), Err(e)) => Err(e),
        (Err(ke), Err(me)) => Err(format!("{ke}; {me}")),
    }
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
