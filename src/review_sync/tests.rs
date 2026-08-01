use super::*;
use crate::orchestrator::clear_review_file;
use std::path::PathBuf;

fn tmp_review_artifact() -> (tempfile::TempDir, PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join(".malvin/logs").join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&artifact, "reviewed\n").unwrap();
    (t, artifact)
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
fn sync_review_file_errors_when_artifact_path_is_not_readable_file() {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join("blocked");
    std::fs::create_dir_all(&artifact).unwrap();
    assert!(sync_review_file(&artifact).is_err());
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
    std::fs::write(&artifact, "reviewed\nsome content").unwrap();
    clear_artifact_review(&artifact).unwrap();
    assert_eq!(std::fs::read_to_string(&artifact).unwrap(), "");
}

#[test]
fn tidy_reviewer_turn_clear_prevents_stale_review_on_sync_attempt() {
    let (_t, artifact) = tmp_review_artifact();
    clear_review_file(&artifact).unwrap();
    let synced = super::sync_review_file_for_attempt(&artifact).expect("sync");
    assert!(
        synced.is_none(),
        "stale review text must not survive artifact clear"
    );
}
