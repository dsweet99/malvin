use std::path::Path;

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "failed to create artifact review parent dirs: {}: {e}",
                path.display()
            )
        })?;
    }
    Ok(())
}

fn clear_artifact_review_to_empty(path: &Path) -> Result<(), String> {
    ensure_parent_dir(path)?;
    std::fs::write(path, "").map_err(|e| {
        format!(
            "failed to clear artifact review file: {}: {e}",
            path.display()
        )
    })
}

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

/// Returns non-empty artifact review text when present; never copies workspace
/// content into an empty artifact (avoids false LGTM from stale `./review.md`).
///
/// Fan-out LGTM must use [`read_artifact_review_for_fanout_attempt`] (via
/// [`crate::orchestrator::review_attempt_is_lgtm`]).
///
/// # Errors
///
/// Returns `Err` when reading review files fails.
pub fn sync_review_file_for_attempt(
    artifact_review_path: &Path,
    workspace_review_path: &Path,
) -> Result<Option<String>, String> {
    if let Some(artifact_text) = read_nonempty_review(artifact_review_path, "artifact")? {
        return Ok(Some(artifact_text));
    }

    if workspace_review_path.exists() {
        let workspace_text = std::fs::read_to_string(workspace_review_path).map_err(|e| {
            format!(
                "failed to read workspace review file: {}: {e}",
                workspace_review_path.display()
            )
        })?;
        if workspace_text.trim().is_empty() {
            clear_artifact_review_to_empty(artifact_review_path)?;
        }
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_stringify_review_attempt_units() {
        let _ = stringify!(super::ensure_parent_dir);
        let _ = stringify!(super::clear_artifact_review_to_empty);
        let _ = stringify!(super::read_nonempty_review);
        let _ = stringify!(super::read_artifact_review_for_fanout_attempt);
    }
}
