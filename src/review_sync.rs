//! Shared `review.md` workspace ↔ run artifact sync and LGTM detection.

use std::io;
use std::path::Path;

/// Check if the given string content represents an LGTM approval.
#[must_use]
pub fn is_lgtm_str(content: &str) -> bool {
    let t = content.trim();
    let t = t.strip_prefix('\u{FEFF}').unwrap_or(t).trim();
    t == "LGTM"
}

#[cfg(test)]
pub fn is_lgtm(review_path: &Path) -> bool {
    std::fs::read_to_string(review_path).is_ok_and(|s| is_lgtm_str(&s))
}

fn clear_artifact_review(artifact_review_path: &Path) -> io::Result<()> {
    if let Some(parent) = artifact_review_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(artifact_review_path, "")
}

/// Syncs the workspace review file to the artifact location, returning the content synced.
///
/// Returns `None` if the workspace file does not exist or is whitespace-only.
pub fn sync_review_file(
    workspace_review_path: &Path,
    artifact_review_path: &Path,
) -> io::Result<Option<String>> {
    if !workspace_review_path.exists() {
        clear_artifact_review(artifact_review_path)?;
        return Ok(None);
    }
    let text = std::fs::read_to_string(workspace_review_path)?;
    if text.trim().is_empty() {
        clear_artifact_review(artifact_review_path)?;
        return Ok(None);
    }
    std::fs::write(artifact_review_path, &text)?;
    Ok(Some(text))
}

/// Sync workspace `review.md` into the run artifact, then return whether the artifact reads as LGTM.
///
/// Returns an error when the workspace file cannot be read or the artifact cannot be written.
///
/// Used by the ACP reviewer pair (`run_reviewer_pair_once` in `ops_body.rs`) so the post-review sequence stays one API surface.
pub fn sync_review_then_is_lgtm(
    workspace_review_path: &Path,
    artifact_review_path: &Path,
) -> io::Result<bool> {
    let content = sync_review_file(workspace_review_path, artifact_review_path)?;
    Ok(content.as_deref().is_some_and(is_lgtm_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn is_lgtm_str_returns_true_for_exact_lgtm() {
        assert!(is_lgtm_str("LGTM"));
        assert!(is_lgtm_str("LGTM\n"));
        assert!(is_lgtm_str("  LGTM  "));
        assert!(is_lgtm_str("\n\tLGTM\n\t"));
    }

    #[test]
    fn is_lgtm_str_with_bom_returns_true() {
        assert!(is_lgtm_str("\u{FEFF}LGTM"));
        assert!(is_lgtm_str("\u{FEFF}LGTM\n"));
    }

    #[test]
    fn is_lgtm_str_returns_false_for_non_lgtm() {
        assert!(!is_lgtm_str(""));
        assert!(!is_lgtm_str("lgtm"));
        assert!(!is_lgtm_str("LGTM!"));
        assert!(!is_lgtm_str("Not LGTM"));
        assert!(!is_lgtm_str("## Concerns\n- issue"));
    }

    #[test]
    fn sync_review_file_returns_content_when_copied() {
        let t = tempfile::tempdir().unwrap();
        let workspace = t.path().join("review.md");
        let artifact = t.path().join("run").join("review.md");
        std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
        std::fs::write(&workspace, "LGTM\n").unwrap();
        let result = sync_review_file(&workspace, &artifact).unwrap();
        assert_eq!(result, Some("LGTM\n".to_string()));
    }

    #[test]
    fn sync_review_file_returns_none_when_workspace_missing() {
        let t = tempfile::tempdir().unwrap();
        let workspace = t.path().join("missing.md");
        let artifact = t.path().join("review.md");
        let result = sync_review_file(&workspace, &artifact).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn sync_review_file_returns_none_when_workspace_empty() {
        let t = tempfile::tempdir().unwrap();
        let workspace = t.path().join("review.md");
        let artifact = t.path().join("review.md");
        std::fs::write(&workspace, "").unwrap();
        let result = sync_review_file(&workspace, &artifact).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn sync_review_then_is_lgtm_true_after_copy() {
        let t = tempfile::tempdir().unwrap();
        let workspace = t.path().join("review.md");
        let artifact = t.path().join("run").join("review.md");
        std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
        let mut f = std::fs::File::create(&workspace).unwrap();
        writeln!(f, "LGTM").unwrap();
        assert!(sync_review_then_is_lgtm(&workspace, &artifact).unwrap());
        assert!(artifact.exists());
    }

    #[test]
    fn sync_review_file_errors_when_artifact_path_is_not_writable_file() {
        let t = tempfile::tempdir().unwrap();
        let workspace = t.path().join("review.md");
        std::fs::write(&workspace, "LGTM\n").unwrap();
        let artifact = t.path().join("blocked");
        std::fs::create_dir_all(&artifact).unwrap();
        assert!(sync_review_file(&workspace, &artifact).is_err());
        assert!(sync_review_then_is_lgtm(&workspace, &artifact).is_err());
    }

    #[test]
    fn sync_review_then_is_lgtm_false_when_workspace_missing() {
        let t = tempfile::tempdir().unwrap();
        let workspace = t.path().join("missing.md");
        let artifact = t.path().join("review.md");
        std::fs::write(&artifact, "nope").unwrap();
        assert!(!sync_review_then_is_lgtm(&workspace, &artifact).unwrap());
    }

    #[test]
    fn sync_review_then_is_lgtm_false_when_workspace_missing_clears_stale_lgtm_artifact() {
        let t = tempfile::tempdir().unwrap();
        let workspace = t.path().join("missing.md");
        let artifact = t.path().join("review.md");
        std::fs::write(&artifact, "LGTM\n").unwrap();
        assert!(!sync_review_then_is_lgtm(&workspace, &artifact).unwrap());
        assert!(!is_lgtm(&artifact));
        assert_eq!(std::fs::read_to_string(&artifact).unwrap(), "");
    }

    #[test]
    fn sync_review_then_is_lgtm_false_when_workspace_whitespace_only_clears_stale_lgtm() {
        let t = tempfile::tempdir().unwrap();
        let workspace = t.path().join("review.md");
        let artifact = t.path().join("run").join("review.md");
        std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
        std::fs::write(&workspace, "  \n\t\n").unwrap();
        std::fs::write(&artifact, "LGTM\n").unwrap();
        assert!(!sync_review_then_is_lgtm(&workspace, &artifact).unwrap());
        assert!(!is_lgtm(&artifact));
        assert_eq!(std::fs::read_to_string(&artifact).unwrap(), "");
    }

    #[test]
    fn is_lgtm_accepts_utf8_bom_prefixed_lgtm() {
        let t = tempfile::tempdir().unwrap();
        let p = t.path().join("r.md");
        let mut f = std::fs::File::create(&p).unwrap();
        f.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
        f.write_all(b"LGTM\n").unwrap();
        assert!(is_lgtm(&p));
    }

    #[test]
    fn clear_artifact_review_creates_parent_dirs_and_empties_file() {
        let t = tempfile::tempdir().unwrap();
        let artifact = t.path().join("nested").join("dir").join("review.md");
        std::fs::write(artifact.parent().unwrap().join("dummy"), "x").ok();
        clear_artifact_review(&artifact).unwrap();
        assert!(artifact.exists());
        assert_eq!(std::fs::read_to_string(&artifact).unwrap(), "");
    }

    #[test]
    fn clear_artifact_review_overwrites_existing_content() {
        let t = tempfile::tempdir().unwrap();
        let artifact = t.path().join("review.md");
        std::fs::write(&artifact, "LGTM\nsome content").unwrap();
        clear_artifact_review(&artifact).unwrap();
        assert_eq!(std::fs::read_to_string(&artifact).unwrap(), "");
    }
}
