use chrono::Utc;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Options for [`create_run_dir`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunDirOptions {
    pub gc: bool,
}

impl Default for RunDirOptions {
    fn default() -> Self {
        Self { gc: true }
    }
}

impl RunDirOptions {
    #[must_use]
    pub const fn without_gc() -> Self {
        Self { gc: false }
    }
}

/// Creates `~/.malvin/logs/<hash>/<timestamp>_<id>/` for `base_dir` (or the current directory).
///
/// # Errors
///
/// Returns [`std::io::Error`] if directory creation fails or unique id allocation exhausts retries.
pub fn create_run_dir(base_dir: Option<&Path>, opts: RunDirOptions) -> std::io::Result<PathBuf> {
    let parent = base_dir.unwrap_or_else(|| Path::new("."));
    let run_root = crate::malvin_logs_root(parent);
    std::fs::create_dir_all(&run_root)?;
    if opts.gc {
        crate::log_gc::prune_logs_before_run(parent);
    }
    create_run_dir_with_id(&run_root, |_| build_identifier())
}

#[must_use]
pub fn build_identifier() -> String {
    let stamp = Utc::now().format("%Y%m%d_%H%M%S");
    let token = random_alnum(8);
    format!("{stamp}_{token}")
}

pub use crate::alnum_id::random_alnum;

fn create_run_dir_with_id(
    run_root: &Path,
    mut generate_id: impl FnMut(usize) -> String,
) -> std::io::Result<PathBuf> {
    let mut tries = 0usize;
    std::fs::create_dir_all(run_root)?;
    while tries < 16 {
        let identifier = generate_id(tries);
        let run_dir = run_root.join(&identifier);
        match std::fs::create_dir(&run_dir) {
            Ok(()) => return Ok(run_dir),
            Err(err) if err.kind() == ErrorKind::AlreadyExists => {
                tries += 1;
            }
            Err(err) => return Err(err),
        }
    }
    Err(std::io::Error::new(
        ErrorKind::AlreadyExists,
        "run directory id collision limit exceeded",
    ))
}

#[cfg(test)]
mod collision_tests {
    use super::*;

    #[test]
    fn create_run_dir_retries_collision_ids() {
        let tmp = tempfile::tempdir().unwrap();
        let run_root = crate::malvin_logs_root(tmp.path());
        std::fs::create_dir_all(&run_root).unwrap();
        std::fs::create_dir_all(run_root.join("aaabbbcc")).unwrap();

        let run_dir = create_run_dir_with_id(&run_root, |attempt| {
            if attempt == 0 {
                "aaabbbcc".to_string()
            } else {
                "aaabbbcd".to_string()
            }
        })
        .unwrap();

        assert_eq!(run_dir, run_root.join("aaabbbcd"));
        assert!(run_dir.is_dir());
    }

    #[test]
    fn create_run_dir_and_build_identifier_smoke() {
        let tmp = tempfile::tempdir().unwrap();
        let id = build_identifier();
        assert!(!id.is_empty());
        let dir = create_run_dir(Some(tmp.path()), RunDirOptions::default()).unwrap();
        assert!(dir.is_dir());
    }
}
