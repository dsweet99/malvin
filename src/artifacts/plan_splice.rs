//! Plan-file boundary parsing and Prompt-3 overwrite for `malvin plan`.

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
    extract_decisions_section, record_user_span_end_after_1a, validate_post_1a, validate_post_1b,
    validate_post_2,
};
pub use plan_splice_boundary::{detect_rerun_user_span_end, find_machine_block_start};
pub use plan_splice_prepare::prepare_plan_file_for_prompt_1a;

pub const BEGIN_MALVIN_MARKER: &str = "BEGIN_MALVIN";
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

pub fn truncate_plan_for_rerun(path: &Path, user_span_end: usize) -> Result<(), PlanFileError> {
    let content = read_plan_file(path)?;
    let truncated = content
        .get(..user_span_end)
        .ok_or_else(|| PlanFileError::Io("user_span_end out of range".to_string()))?
        .to_string();
    write_plan_file_atomic(path, &truncated)
}

pub(crate) fn ensure_user_span_trailing_newlines(spliced: &mut String) {
    if !spliced.ends_with('\n') && !spliced.is_empty() {
        spliced.push('\n');
    }
    if !spliced.is_empty()
        && !spliced.ends_with("\n\n")
        && !spliced.ends_with('\n')
    {
        spliced.push('\n');
    }
}

pub(crate) fn append_machine_block(spliced: &mut String, fenced_body: &str) {
    if !spliced.is_empty() && !spliced.ends_with('\n') {
        spliced.push('\n');
    }
    if !spliced.is_empty()
        && !spliced.ends_with("\n\n")
        && !spliced.ends_with("\r\n\r\n")
    {
        spliced.push('\n');
    }
    spliced.push_str("---\n");
    spliced.push_str(BEGIN_MALVIN_MARKER);
    spliced.push('\n');
    spliced.push_str(fenced_body.trim_end());
    if !spliced.ends_with('\n') {
        spliced.push('\n');
    }
}

pub fn overwrite_plan_file(path: &Path, body: &str) -> Result<(), PlanFileError> {
    let mut content = body.trim_end().to_string();
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    write_plan_file_atomic(path, &content)
}

pub fn splice_plan_file(
    path: &Path,
    user_span_end: usize,
    fenced_body: &str,
) -> Result<(), PlanFileError> {
    let content = read_plan_file(path)?;
    let user = content
        .get(..user_span_end)
        .ok_or_else(|| PlanFileError::Io("user_span_end out of range".to_string()))?;
    let mut spliced = user.to_string();
    ensure_user_span_trailing_newlines(&mut spliced);
    append_machine_block(&mut spliced, fenced_body);
    write_plan_file_atomic(path, &spliced)
}

pub fn snapshot_plan_artifact(run_dir: &Path, name: &str, source: &Path) -> Result<PathBuf, PlanFileError> {
    let dest = run_dir.join(name);
    std::fs::copy(source, &dest).map_err(|e| {
        PlanFileError::Io(format!("plan snapshot {name}: {e}"))
    })?;
    Ok(dest)
}

pub fn prepare_plan_file_for_run(path: &Path) -> Result<Option<usize>, PlanFileError> {
    let content = read_plan_file(path)?;
    match detect_rerun_user_span_end(&content)? {
        None => Ok(None),
        Some(user_span_end) => {
            truncate_plan_for_rerun(path, user_span_end)?;
            Ok(Some(user_span_end))
        }
    }
}

#[cfg(test)]
mod private_fn_coverage {
    use super::*;

    #[test]
    fn ensure_user_span_trailing_newlines_noop_for_empty() {
        let mut s = String::new();
        ensure_user_span_trailing_newlines(&mut s);
        assert!(s.is_empty());
    }

    #[test]
    fn ensure_user_span_trailing_newlines_adds_newline_when_missing() {
        let mut s = "user".to_string();
        ensure_user_span_trailing_newlines(&mut s);
        assert_eq!(s, "user\n");
    }

    #[test]
    fn append_machine_block_appends_markers_and_body() {
        let mut s = "user\n\n".to_string();
        append_machine_block(&mut s, "# Plan");
        assert!(s.contains("---\nBEGIN_MALVIN\n# Plan\n"));
    }
}

#[cfg(test)]
mod plan_file_io_error {
    use std::io::{Error, ErrorKind};

    pub fn plan_file_io_error(msg: &str) -> Error {
        Error::new(ErrorKind::InvalidInput, msg.to_string())
    }
}

#[cfg(test)]
#[path = "plan_splice_tests.rs"]
mod plan_splice_tests;

#[cfg(test)]
#[path = "plan_splice_io_tests.rs"]
mod plan_splice_io_tests;
