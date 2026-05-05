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

/// Copies workspace `review.md` into the run artifact path for one review attempt.
///
/// # Errors
///
/// Returns `Err` when reading the workspace file or writing the artifact fails.
pub fn sync_review_file_for_attempt(
    artifact_review_path: &Path,
    workspace_review_path: &Path,
) -> Result<Option<String>, String> {
    if workspace_review_path.exists() {
        let workspace_text = std::fs::read_to_string(workspace_review_path).map_err(|e| {
            format!(
                "failed to read workspace review file: {}: {e}",
                workspace_review_path.display()
            )
        })?;
        if workspace_text.trim().is_empty() {
            clear_artifact_review_to_empty(artifact_review_path)?;
            return Ok(None);
        }
        ensure_parent_dir(artifact_review_path)?;
        std::fs::write(artifact_review_path, &workspace_text).map_err(|e| {
            format!(
                "failed to sync workspace review into artifact: {}: {e}",
                artifact_review_path.display()
            )
        })?;
        return Ok(Some(workspace_text));
    }

    if artifact_review_path.exists() {
        let artifact_text = std::fs::read_to_string(artifact_review_path).map_err(|e| {
            format!(
                "failed to read artifact review file: {}: {e}",
                artifact_review_path.display()
            )
        })?;
        if !artifact_text.trim().is_empty() {
            return Ok(Some(artifact_text));
        }
    }

    Ok(None)
}
