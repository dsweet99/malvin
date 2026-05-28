use std::path::Path;

pub fn read_nonempty_review(path: &Path, label: &str) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path).map_err(|e| {
        if label.is_empty() {
            format!("failed to read review file: {}: {e}", path.display())
        } else {
            format!(
                "failed to read {label} review file: {}: {e}",
                path.display()
            )
        }
    })?;
    if text.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

/// Non-empty artifact `review.md` after fan-out + `review_write` (no workspace fallback).
///
/// # Errors
///
/// Returns `Err` when reading the artifact review file fails.
pub fn read_artifact_review_for_fanout_attempt(
    artifact_review_path: &Path,
) -> Result<Option<String>, String> {
    read_nonempty_review(artifact_review_path, "artifact")
}

/// Returns non-empty artifact review text when present.
///
/// Fan-out LGTM must use [`read_artifact_review_for_fanout_attempt`].
///
/// # Errors
///
/// Returns `Err` when reading the artifact review file fails.
pub fn sync_review_file_for_attempt(
    artifact_review_path: &Path,
) -> Result<Option<String>, String> {
    read_nonempty_review(artifact_review_path, "artifact")
}

#[cfg(test)]
mod tests {
    #[test]
    fn sync_review_file_for_attempt_returns_none_when_artifact_missing() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifact = tmp.path().join("nested/review.md");
        let text = super::sync_review_file_for_attempt(&artifact).expect("sync");
        assert!(text.is_none());
        assert!(!artifact.exists());
    }

    #[test]
    fn read_nonempty_review_returns_none_for_missing_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let missing = tmp.path().join("missing.md");
        assert_eq!(
            super::read_nonempty_review(&missing, "artifact").expect("read"),
            None
        );
        assert_eq!(
            super::read_artifact_review_for_fanout_attempt(&missing).expect("fanout read"),
            None
        );
    }
}
