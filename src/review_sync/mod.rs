//! Shared `review.md` workspace ↔ run artifact sync and LGTM detection.

mod attempt;

pub use attempt::{read_artifact_review_for_fanout_attempt, sync_review_file_for_attempt};

#[must_use]
pub fn is_lgtm_str(content: &str) -> bool {
    let t = content.trim();
    let t = t.strip_prefix('\u{FEFF}').unwrap_or(t).trim();
    t == "LGTM"
}

#[cfg(test)]
mod is_lgtm_path {
    #![allow(clippy::must_use_candidate)]

    use std::path::Path;

    pub fn is_lgtm(review_path: &Path) -> bool {
        std::fs::read_to_string(review_path).is_ok_and(|s| super::is_lgtm_str(&s))
    }
}

#[cfg(test)]
pub use is_lgtm_path::is_lgtm;

#[cfg(test)]
fn clear_artifact_review(artifact_review_path: &std::path::Path) -> std::io::Result<()> {
    if let Some(parent) = artifact_review_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(artifact_review_path, "")
}

#[cfg(test)]
/// Test helper: mirrors production sync semantics using [`std::io::Error`].
///
/// # Errors
///
/// Returns [`std::io::Error`] when reading or writing review files fails.
pub fn sync_review_file(
    workspace_review_path: &std::path::Path,
    artifact_review_path: &std::path::Path,
) -> std::io::Result<Option<String>> {
    sync_review_file_for_attempt(artifact_review_path, workspace_review_path)
        .map_err(std::io::Error::other)
}

#[cfg(test)]
fn sync_review_then_is_lgtm(
    workspace_review_path: &std::path::Path,
    artifact_review_path: &std::path::Path,
) -> std::io::Result<bool> {
    let content = sync_review_file(workspace_review_path, artifact_review_path)?;
    Ok(content.as_deref().is_some_and(is_lgtm_str))
}

#[cfg(test)]
mod fanout_read_tests;

#[cfg(test)]
mod tests;
