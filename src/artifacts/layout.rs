use std::path::PathBuf;

use super::{LITE_DIR, MAIN_DIR, RunArtifacts, TRACE_JSONL};

pub struct AgentDirs<'a>(pub &'a RunArtifacts);

impl AgentDirs<'_> {
    #[must_use]
    pub fn main(&self) -> PathBuf {
        self.0.run_dir.join(MAIN_DIR)
    }

    #[must_use]
    pub fn lite(&self) -> PathBuf {
        self.0.run_dir.join(LITE_DIR)
    }

    #[must_use]
    pub fn trace(&self) -> PathBuf {
        self.main().join(TRACE_JSONL)
    }

    #[must_use]
    pub fn lite_trace(&self) -> PathBuf {
        self.lite().join(TRACE_JSONL)
    }

    pub fn create(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(self.main())?;
        std::fs::create_dir_all(self.lite())
    }
}

#[cfg(test)]
mod tests {
    use super::AgentDirs;
    use crate::artifacts::create_run_artifacts_from_text;

    #[test]
    fn run_creates_main_and_lite_dirs_with_separate_traces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = create_run_artifacts_from_text("hi", Some(tmp.path())).expect("artifacts");
        let dirs = AgentDirs(&artifacts);
        assert!(dirs.main().is_dir());
        assert!(dirs.lite().is_dir());
        assert_eq!(dirs.trace(), artifacts.run_dir.join("main/trace.jsonl"));
        assert_eq!(
            dirs.lite_trace(),
            artifacts.run_dir.join("lite/trace.jsonl")
        );
        assert_eq!(
            artifacts.stdout_log_path(),
            artifacts.run_dir.join("main/stdout.log")
        );
    }
}
