//! Plan-file overwrite and Prompt-3 commit for `malvin plan`.

use std::path::{Path, PathBuf};

#[path = "plan_metadata.rs"]
mod plan_metadata;
#[path = "plan_validate.rs"]
mod plan_validate;
#[path = "plan_splice_boundary.rs"]
mod plan_splice_boundary;
#[path = "plan_splice_prepare.rs"]
mod plan_splice_prepare;
#[path = "plan_splice_fence.rs"]
mod plan_splice_fence;

pub use plan_metadata::{PlanRunMetadata, read_plan_metadata, write_plan_metadata};
pub use plan_splice_fence::extract_fenced_markdown_block;
pub use plan_validate::{
    extract_decisions_section, validate_post_1a, validate_post_1b, validate_post_2,
};
pub use plan_splice_boundary::{
    detect_rerun_user_span_end, find_machine_block_start, is_interrupted_machine_plan,
    plan_user_sidecar_path, remove_plan_user_sidecar, restore_interrupted_plan,
};

/// Legacy delimiter name retained for tests and interrupted-run recovery of older plan files.
pub const BEGIN_MALVIN_MARKER: &str = plan_splice_boundary::LEGACY_BEGIN_MALVIN_MARKER;
pub use plan_splice_prepare::prepare_plan_file_for_prompt_1a;

pub(crate) const PLAN_METADATA_FILE: &str = "plan_metadata.json";

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PlanFileError {
    #[error("plan file has duplicate BEGIN_MALVIN marker lines; re-run requires a single machine block")]
    DuplicateBeginMalvinMarkers,
    #[error("plan file has malformed machine block delimiter; expected newline-delimited --- then BEGIN_MALVIN")]
    MalformedMachineBlockDelimiter,
    #[error("plan file missing required section {0}")]
    MissingSection(&'static str),
    #[error("plan prompt 3 response missing fenced markdown block")]
    MissingFencedBlock,
    #[error("{0}")]
    Io(String),
}

impl From<std::io::Error> for PlanFileError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

pub fn read_plan_file(path: &Path) -> Result<String, PlanFileError> {
    std::fs::read_to_string(path).map_err(PlanFileError::from)
}

pub fn write_plan_file_atomic(path: &Path, content: &str) -> Result<(), PlanFileError> {
    let parent = path.parent().ok_or_else(|| {
        PlanFileError::Io("plan path has no parent directory".to_string())
    })?;
    std::fs::create_dir_all(parent)?;
    let tmp = path.with_extension("md.tmp");
    std::fs::write(&tmp, content)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

pub(crate) fn truncate_plan_for_rerun(path: &Path, user_span_end: usize) -> Result<(), PlanFileError> {
    let content = read_plan_file(path)?;
    let truncated = content
        .get(..user_span_end)
        .ok_or_else(|| PlanFileError::Io("user_span_end out of range".to_string()))?
        .to_string();
    write_plan_file_atomic(path, &truncated)
}

pub fn overwrite_plan_file(path: &Path, body: &str) -> Result<(), PlanFileError> {
    let mut content = body.trim_end().to_string();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    write_plan_file_atomic(path, &content)
}

pub fn snapshot_plan_artifact(run_dir: &Path, name: &str, source: &Path) -> Result<PathBuf, PlanFileError> {
    let dest = run_dir.join(name);
    std::fs::copy(source, &dest).map_err(|e| {
        PlanFileError::Io(format!("plan snapshot {name}: {e}"))
    })?;
    Ok(dest)
}

pub fn prepare_plan_file_for_run(path: &Path) -> Result<bool, PlanFileError> {
    restore_interrupted_plan(path)
}

#[cfg(test)]
#[path = "plan_splice_kiss_cov_test.rs"]
mod plan_splice_kiss_cov_test;

mod plan_file_io_error {
    use std::io::{Error, ErrorKind};

    pub fn plan_file_io_error(msg: &str) -> Error {
        Error::new(ErrorKind::InvalidInput, msg.to_string())
    }
}
