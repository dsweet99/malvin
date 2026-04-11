//! Run directories and log paths.

use chrono::Utc;
use rand::Rng;
use std::path::{Path, PathBuf};

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
}

/// Copy `plan_source` into a fresh run directory under `base_dir`/`_malvin`/…
///
/// # Errors
///
/// Returns an I/O error if directories cannot be created or the plan cannot be copied.
pub fn create_run_artifacts(plan_source: &Path, base_dir: Option<&Path>) -> std::io::Result<RunArtifacts> {
    let run_dir = create_run_dir(base_dir)?;
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
    let work_dir = base_dir
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let run_dir = create_run_dir(base_dir)?;
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
    let work_dir = base_dir
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let run_dir = create_run_dir(base_dir)?;
    let request_target = run_dir.join("request.md");
    std::fs::write(&request_target, request_text)?;
    Ok(RunArtifacts {
        run_dir,
        plan_path: request_target,
        work_dir,
    })
}

fn work_dir_for_path(path: &Path) -> PathBuf {
    path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn resolve_at_file(rest: &str) -> Result<(String, PathBuf), String> {
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

fn create_run_dir(base_dir: Option<&Path>) -> std::io::Result<PathBuf> {
    let parent = base_dir.unwrap_or_else(|| Path::new("."));
    let run_root = parent.join("_malvin");
    std::fs::create_dir_all(&run_root)?;
    let identifier = build_identifier();
    let run_dir = run_root.join(&identifier);
    std::fs::create_dir(&run_dir)?;
    Ok(run_dir)
}

fn build_identifier() -> String {
    let stamp = Utc::now().format("%Y%m%d_%H%M%S");
    let token = random_alnum(8);
    format!("{stamp}_{token}")
}

fn random_alnum(len: usize) -> String {
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| {
            let i = rng.gen_range(0..ALPHABET.len());
            ALPHABET[i] as char
        })
        .collect()
}

#[cfg(test)]
mod kiss_refs {
    #[test]
    fn stringify_private_helpers() {
        let _ = stringify!(super::create_run_dir);
        let _ = stringify!(super::build_identifier);
        let _ = stringify!(super::random_alnum);
        let _ = stringify!(super::create_kpop_run_artifacts);
        let _ = stringify!(super::resolve_user_request);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_path_sanitizes_slashes() {
        let r = RunArtifacts {
            run_dir: PathBuf::from("/tmp/run"),
            plan_path: PathBuf::from("/tmp/run/plan.md"),
            work_dir: PathBuf::from("/work"),
        };
        assert_eq!(
            r.log_path("a/b").file_name(),
            Some(std::ffi::OsStr::new("a_b.log"))
        );
    }

    #[test]
    fn log_path_sanitizes_backslashes() {
        let r = RunArtifacts {
            run_dir: PathBuf::from("/tmp/run"),
            plan_path: PathBuf::from("/tmp/run/plan.md"),
            work_dir: PathBuf::from("/work"),
        };
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
