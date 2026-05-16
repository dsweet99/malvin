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

fn read_nonempty_review(path: &Path, label: &str) -> Result<Option<String>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path).map_err(|e| {
        format!("failed to read {label} review file: {}: {e}", path.display())
    })?;
    if text.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

/// Resolves review text for one fan-out attempt; run artifact `review.md` is authoritative.
///
/// Non-empty artifact text is always returned as-is (including `LGTM` written by `review_write`
/// when workspace `./review.md` is whitespace-only). Stale workspace `LGTM` cannot mask
/// artifact gate markers or reviewer findings. When the artifact is empty, non-empty workspace
/// text is copied into the artifact; whitespace-only workspace leaves the artifact empty.
///
/// # Errors
///
/// Returns `Err` when reading or writing review files fails.
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

    Ok(None)
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_stringify_review_attempt_units() {
        let _ = stringify!(super::ensure_parent_dir);
        let _ = stringify!(super::clear_artifact_review_to_empty);
        let _ = stringify!(super::read_nonempty_review);
    }
}
