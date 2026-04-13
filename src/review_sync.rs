//! Shared `review.md` workspace ↔ run artifact sync and LGTM detection.

use std::io;
use std::path::Path;

#[must_use]
pub fn is_lgtm(review_path: &Path) -> bool {
    std::fs::read_to_string(review_path)
        .map(|s| {
            let t = s.trim();
            let t = t.strip_prefix('\u{FEFF}').unwrap_or(t).trim();
            t == "LGTM"
        })
        .unwrap_or(false)
}

fn clear_artifact_review(artifact_review_path: &Path) -> io::Result<()> {
    if let Some(parent) = artifact_review_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(artifact_review_path, "")
}

pub fn sync_review_file(
    workspace_review_path: &Path,
    artifact_review_path: &Path,
) -> io::Result<()> {
    if !workspace_review_path.exists() {
        // Do not leave a stale artifact LGTM when the workspace has no review file.
        clear_artifact_review(artifact_review_path)?;
        return Ok(());
    }
    let text = std::fs::read_to_string(workspace_review_path)?;
    if text.trim().is_empty() {
        clear_artifact_review(artifact_review_path)?;
        return Ok(());
    }
    std::fs::write(artifact_review_path, text)
}

/// Sync workspace `review.md` into the run artifact, then return whether the artifact reads as LGTM.
///
/// Returns an error when the workspace file cannot be read or the artifact cannot be written.
///
/// Used by the ACP reviewer pair (`run_reviewer_pair_once` in `ops_body.inc`) so the post-review sequence stays one API surface.
pub fn sync_review_then_is_lgtm(
    workspace_review_path: &Path,
    artifact_review_path: &Path,
) -> io::Result<bool> {
    sync_review_file(workspace_review_path, artifact_review_path)?;
    Ok(is_lgtm(artifact_review_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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
}
