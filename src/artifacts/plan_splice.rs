//! Plan-file boundary parsing and Prompt-3 splice for `malvin plan`.

use std::path::{Path, PathBuf};

#[path = "plan_metadata.rs"]
mod plan_metadata;
#[path = "plan_validate.rs"]
mod plan_validate;

pub use plan_metadata::{PlanRunMetadata, read_plan_metadata, write_plan_metadata};
pub use plan_validate::{
    extract_decisions_section, record_user_span_end_after_1a, validate_post_1a, validate_post_1b,
    validate_post_2,
};

pub const BEGIN_MALVIN_MARKER: &str = "BEGIN_MALVIN";
pub(crate) const PLAN_METADATA_FILE: &str = "plan_metadata.json";

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PlanFileError {
    #[error("plan file has ambiguous BEGIN_MALVIN markers; re-run requires a single machine block")]
    AmbiguousMarkers,
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

/// Byte offset before the machine block (`---` line), if exactly one unambiguous block exists.
#[must_use]
pub fn find_machine_block_start(content: &str) -> Option<usize> {
    let marker = format!("\n---\n{BEGIN_MALVIN_MARKER}");
    let at_start = format!("---\n{BEGIN_MALVIN_MARKER}");
    if content.starts_with(&at_start) {
        return Some(0);
    }
    if let Some(idx) = content.find(&marker) {
        let user = &content[..idx];
        if user.contains(BEGIN_MALVIN_MARKER) {
            return None;
        }
        return Some(idx + 1);
    }
    None
}

/// Returns `user_span_end` when a single machine block is present; errors when markers are ambiguous.
pub fn detect_rerun_user_span_end(content: &str) -> Result<Option<usize>, PlanFileError> {
    let count = content.matches(BEGIN_MALVIN_MARKER).count();
    if count == 0 {
        return Ok(None);
    }
    if count > 1 {
        return Err(PlanFileError::AmbiguousMarkers);
    }
    find_machine_block_start(content)
        .ok_or(PlanFileError::AmbiguousMarkers)
        .map(Some)
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

/// Extract inner text from the first ```markdown or ``` fenced block in `response`.
pub fn extract_fenced_markdown_block(response: &str) -> Result<String, PlanFileError> {
    let trimmed = response.trim();
    for fence in ["```markdown", "```md", "```"] {
        if let Some(body) = extract_fence_body(trimmed, fence) {
            if !body.trim().is_empty() {
                return Ok(body.trim().to_string());
            }
        }
    }
    Err(PlanFileError::MissingFencedBlock)
}

fn extract_fence_body(text: &str, fence: &str) -> Option<String> {
    let start = text.find(fence)?;
    if fence == "```" {
        let after_marker = &text[start + fence.len()..];
        if after_marker.starts_with("markdown") || after_marker.starts_with("md") {
            return None;
        }
    }
    let mut after_open = &text[start + fence.len()..];
    if let Some(stripped) = after_open.strip_prefix('\r') {
        after_open = stripped;
    }
    after_open = after_open.strip_prefix('\n').unwrap_or(after_open);
    let close = after_open.find("\n```").or_else(|| after_open.rfind("```"))?;
    Some(after_open[..close].to_string())
}

fn ensure_user_span_trailing_newlines(spliced: &mut String) {
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

fn append_machine_block(spliced: &mut String, fenced_body: &str) {
    spliced.push_str("\n---\n");
    spliced.push_str(BEGIN_MALVIN_MARKER);
    spliced.push('\n');
    spliced.push_str(fenced_body.trim_end());
    if !spliced.ends_with('\n') {
        spliced.push('\n');
    }
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
    fn extract_fence_body_skips_plain_fence_with_markdown_prefix() {
        assert!(extract_fence_body("```markdown\nx\n```", "```").is_none());
    }

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
