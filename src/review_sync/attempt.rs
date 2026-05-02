use std::path::Path;

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
        if !workspace_text.trim().is_empty() {
            std::fs::write(artifact_review_path, &workspace_text).map_err(|e| {
                format!(
                    "failed to sync workspace review into artifact: {}: {e}",
                    artifact_review_path.display()
                )
            })?;
            return Ok(Some(workspace_text));
        }
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
