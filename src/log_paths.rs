//! Human-readable paths for run log location hints in CLI output.

use std::path::Path;

/// `./`-prefixed path to `run_dir` relative to the process current directory when possible.
///
/// If `run_dir` is not under the current directory, returns an absolute display path.
///
/// # Errors
///
/// Returns a string error when the current directory or `run_dir` cannot be resolved.
pub fn format_logs_dir(run_dir: &Path) -> Result<String, String> {
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    let cwd_abs = cwd.canonicalize().map_err(|e| e.to_string())?;
    let run_abs = run_dir.canonicalize().map_err(|e| e.to_string())?;
    Ok(run_abs.strip_prefix(&cwd_abs).map_or_else(
        |_| run_abs.display().to_string(),
        |p| format!("./{}", p.display()),
    ))
}

#[cfg(test)]
mod tests {
    use crate::test_utils::test_env_lock;

    use super::format_logs_dir;

    #[test]
    fn relative_when_run_dir_is_under_cwd() {
        let _g = test_env_lock();
        let old = std::env::current_dir().unwrap();
        let base = tempfile::tempdir().unwrap();
        let run = base.path().join("nest").join("run");
        std::fs::create_dir_all(&run).unwrap();
        std::env::set_current_dir(base.path()).unwrap();
        let out = format_logs_dir(&run).unwrap();
        std::env::set_current_dir(&old).unwrap();
        assert_eq!(out, "./nest/run");
    }

    #[test]
    fn absolute_when_run_dir_not_under_cwd() {
        let _g = test_env_lock();
        let old = std::env::current_dir().unwrap();
        let cwd_tmp = tempfile::tempdir().unwrap();
        let run_tmp = tempfile::tempdir().unwrap();
        let run = run_tmp.path().join("outside");
        std::fs::create_dir_all(&run).unwrap();
        std::env::set_current_dir(cwd_tmp.path()).unwrap();
        let out = format_logs_dir(&run).unwrap();
        std::env::set_current_dir(&old).unwrap();
        assert!(
            out.starts_with('/'),
            "expected absolute path when run is not under cwd, got {out:?}"
        );
        assert!(out.contains("outside"));
    }

    #[test]
    fn errors_when_run_dir_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("nope").join("run");
        let err = format_logs_dir(&p).unwrap_err();
        assert!(!err.is_empty());
    }
}
