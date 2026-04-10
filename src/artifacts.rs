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
    let parent = base_dir.unwrap_or_else(|| Path::new("."));
    let run_root = parent.join("_malvin");
    std::fs::create_dir_all(&run_root)?;
    let identifier = build_identifier();
    let run_dir = run_root.join(&identifier);
    std::fs::create_dir(&run_dir)?;
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
        let _ = stringify!(super::build_identifier);
        let _ = stringify!(super::random_alnum);
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
}
