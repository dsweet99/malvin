//! Shared `review.md` workspace ↔ run artifact sync and LGTM detection.

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

pub fn sync_review_file(workspace_review_path: &Path, artifact_review_path: &Path) {
    if !workspace_review_path.exists() {
        return;
    }
    if let Ok(text) = std::fs::read_to_string(workspace_review_path) {
        if text.trim().is_empty() {
            return;
        }
        let _ = std::fs::write(artifact_review_path, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

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
