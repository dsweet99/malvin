//! Run directories and log paths.

mod grounding_backup;
pub mod run_id;
mod startup_tag;

pub use grounding_backup::{
    GroundingBackup, backup_workspace_grounding_if_present, restore_workspace_grounding,
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
mod tests {
    use super::*;

    #[test]
    fn log_path_sanitizes_slashes_and_backslashes() {
        let r = RunArtifacts {
            run_dir: PathBuf::from("/tmp/run"),
            plan_path: PathBuf::from("/tmp/run/plan.md"),
            work_dir: PathBuf::from("/work"),
        };
        assert_eq!(
            r.log_path("a/b").file_name(),
            Some(std::ffi::OsStr::new("a_b.log"))
        );
        assert_eq!(
            r.log_path("a\\b").file_name(),
            Some(std::ffi::OsStr::new("a_b.log"))
        );
    }

    #[test]
    fn create_run_artifacts_relative_plan_uses_dot_work_dir() {
        let _guard = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(tmp.path()).unwrap();
        std::fs::write("plan.md", "restated request").unwrap();

        let art = create_run_artifacts(Path::new("plan.md"), None).unwrap();
        std::env::set_current_dir(old_cwd).unwrap();
        assert_eq!(art.work_dir, PathBuf::from("."));
    }

    #[test]
    fn create_run_artifacts_from_text_uses_base_dir_as_work_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let art = create_run_artifacts_from_text("prompt", Some(tmp.path())).unwrap();
        assert_eq!(art.work_dir, tmp.path());
        assert_eq!(std::fs::read_to_string(&art.plan_path).unwrap(), "prompt");
    }

    #[test]
    fn resolve_user_request_literal_uses_dot_work_dir_and_trims() {
        let (text, wd) = resolve_user_request("  hello world  ").unwrap();
        assert_eq!(text, "hello world");
        assert_eq!(wd, PathBuf::from("."));
    }

    #[test]
    fn resolve_user_request_at_file_reads_contents_and_parent_work_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("note.md");
        std::fs::write(&f, "line1\n").unwrap();
        let arg = format!("@{}", f.display());
        let (text, wd) = resolve_user_request(&arg).unwrap();
        assert_eq!(text, "line1\n");
        assert_eq!(wd, tmp.path());
    }

    #[test]
    fn resolve_user_request_at_missing_file_errors() {
        let err = resolve_user_request("@/no/such/file/plan_zz.md").unwrap_err();
        assert!(err.contains("does not exist"), "unexpected err: {err}");
    }

    #[test]
    fn resolve_user_request_at_empty_path_errors() {
        let err = resolve_user_request("@").unwrap_err();
        assert_eq!(err, "Empty path after `@`.");
    }
}
