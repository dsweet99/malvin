use super::*;
use crate::orchestrator::clear_review_file;
use std::io::Write;
use std::path::PathBuf;

fn tmp_lgtm_artifact() -> (tempfile::TempDir, PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join(".malvin/logs").join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&artifact, "LGTM\n").unwrap();
    (t, artifact)
}

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
fn sync_review_file_for_attempt_returns_none_when_artifact_missing() {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join("run").join("review.md");
    let out = super::sync_review_file_for_attempt(&artifact).unwrap();
    assert_eq!(out, None);
}

#[test]
fn sync_review_file_for_attempt_returns_artifact_text_when_present() {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&artifact, "Checks do not pass\n").unwrap();
    let out = super::sync_review_file_for_attempt(&artifact).unwrap();
    assert_eq!(out.as_deref(), Some("Checks do not pass\n"));
}

#[test]
fn sync_review_file_for_attempt_returns_none_when_artifact_empty() {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&artifact, "  \n\t\n").unwrap();
    let out = super::sync_review_file_for_attempt(&artifact).unwrap();
    assert_eq!(out, None);
}

#[test]
fn sync_review_file_returns_none_when_artifact_missing() {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join("run").join("review.md");
    let result = sync_review_file(&artifact).unwrap();
    assert_eq!(result, None);
}

#[test]
fn sync_review_file_returns_none_when_artifact_empty() {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join("review.md");
    std::fs::write(&artifact, "").unwrap();
    let result = sync_review_file(&artifact).unwrap();
    assert_eq!(result, None);
}

#[test]
fn sync_review_then_is_lgtm_false_when_artifact_missing() {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    assert!(!sync_review_then_is_lgtm(&artifact).unwrap());
}

#[test]
fn sync_review_file_errors_when_artifact_path_is_not_readable_file() {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join("blocked");
    std::fs::create_dir_all(&artifact).unwrap();
    assert!(sync_review_file(&artifact).is_err());
    assert!(sync_review_then_is_lgtm(&artifact).is_err());
}

#[test]
fn sync_review_then_is_lgtm_true_when_artifact_lgtm() {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join("review.md");
    std::fs::write(&artifact, "LGTM\n").unwrap();
    assert!(sync_review_then_is_lgtm(&artifact).unwrap());
    assert!(is_lgtm(&artifact));
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

#[test]
fn tidy_reviewer_turn_clear_prevents_stale_lgtm_on_sync_attempt() {
    let (_t, artifact) = tmp_lgtm_artifact();
    clear_review_file(&artifact).unwrap();
    let synced = super::sync_review_file_for_attempt(&artifact).expect("sync");
    assert!(
        !synced.as_deref().is_some_and(is_lgtm_str),
        "stale LGTM must not survive artifact clear"
    );
}
