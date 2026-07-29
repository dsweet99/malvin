//! Run artifact `review.md` read helpers (non-empty sync / fan-out reads).

mod attempt;

pub use attempt::{read_artifact_review_for_fanout_attempt, sync_review_file_for_attempt};

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
    artifact_review_path: &std::path::Path,
) -> std::io::Result<Option<String>> {
    sync_review_file_for_attempt(artifact_review_path).map_err(std::io::Error::other)
}

#[cfg(test)]
mod fanout_read_tests;

#[cfg(test)]
mod tests;
