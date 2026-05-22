use super::*;
use crate::orchestrator::clear_review_file;
use std::io::Write;
use std::path::PathBuf;

fn tmp_lgtm_artifact_and_workspace() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join("_malvin").join("run").join("review.md");
    let workspace = t.path().join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&artifact, "LGTM\n").unwrap();
    std::fs::write(&workspace, "LGTM\n").unwrap();
    (t, artifact, workspace)
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
fn sync_review_file_for_attempt_does_not_promote_workspace_lgtm_to_empty_artifact() {
    let t = tempfile::tempdir().unwrap();
    let workspace = t.path().join("review.md");
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&workspace, "LGTM\n").unwrap();
    let out = super::sync_review_file_for_attempt(&artifact, &workspace).unwrap();
    assert_eq!(out, None);
    assert!(
        !artifact.exists()
            || std::fs::read_to_string(&artifact)
                .unwrap()
                .trim()
                .is_empty(),
        "workspace LGTM must not be copied into empty artifact"
    );
}

#[test]
fn sync_review_file_for_attempt_prefers_artifact_over_stale_workspace_lgtm() {
    let t = tempfile::tempdir().unwrap();
    let workspace = t.path().join("review.md");
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&workspace, "LGTM\n").unwrap();
    std::fs::write(&artifact, "Checks do not pass\n").unwrap();
    let out = super::sync_review_file_for_attempt(&artifact, &workspace).unwrap();
    assert_eq!(out.as_deref(), Some("Checks do not pass\n"));
    assert_eq!(
        std::fs::read_to_string(&artifact).unwrap(),
        "Checks do not pass\n"
    );
}

#[test]
fn sync_review_file_for_attempt_falls_back_to_nonempty_artifact() {
    let t = tempfile::tempdir().unwrap();
    let workspace = t.path().join("missing.md");
    let artifact = t.path().join("review.md");
    std::fs::write(&artifact, "LGTM\n").unwrap();
    let out = super::sync_review_file_for_attempt(&artifact, &workspace).unwrap();
    assert_eq!(out.as_deref(), Some("LGTM\n"));
}

#[test]
fn sync_review_file_for_attempt_preserves_artifact_lgtm_when_workspace_whitespace_only() {
    let t = tempfile::tempdir().unwrap();
    let workspace = t.path().join("review.md");
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&artifact, "LGTM\n").unwrap();
    std::fs::write(&workspace, "\n").unwrap();
    let out = super::sync_review_file_for_attempt(&artifact, &workspace).unwrap();
    assert!(
        out.as_deref().is_some_and(is_lgtm_str),
        "fresh artifact LGTM after review_write must survive whitespace-only workspace file"
    );
    assert_eq!(
        std::fs::read_to_string(&artifact).unwrap(),
        "LGTM\n",
        "sync must not clear artifact LGTM when workspace is whitespace-only"
    );
}

#[test]
fn sync_review_file_for_attempt_whitespace_workspace_yields_none_when_artifact_empty() {
    let t = tempfile::tempdir().unwrap();
    let workspace = t.path().join("review.md");
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&workspace, "  \n\t\n").unwrap();
    let out = super::sync_review_file_for_attempt(&artifact, &workspace).unwrap();
    assert_eq!(out, None);
    assert!(
        !artifact.exists() || std::fs::read_to_string(&artifact).unwrap().is_empty(),
        "whitespace workspace with empty artifact must not produce review text"
    );
}

#[test]
fn sync_review_file_returns_none_when_artifact_empty_and_workspace_lgtm() {
    let t = tempfile::tempdir().unwrap();
    let workspace = t.path().join("review.md");
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&workspace, "LGTM\n").unwrap();
    let result = sync_review_file(&workspace, &artifact).unwrap();
    assert_eq!(result, None);
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
fn sync_review_then_is_lgtm_false_when_only_workspace_lgtm() {
    let t = tempfile::tempdir().unwrap();
    let workspace = t.path().join("review.md");
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    let mut f = std::fs::File::create(&workspace).unwrap();
    writeln!(f, "LGTM").unwrap();
    assert!(!sync_review_then_is_lgtm(&workspace, &artifact).unwrap());
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
fn sync_review_then_is_lgtm_true_when_artifact_lgtm_even_if_workspace_missing() {
    let t = tempfile::tempdir().unwrap();
    let workspace = t.path().join("missing.md");
    let artifact = t.path().join("review.md");
    std::fs::write(&artifact, "LGTM\n").unwrap();
    assert!(sync_review_then_is_lgtm(&workspace, &artifact).unwrap());
    assert!(is_lgtm(&artifact));
}

#[test]
fn sync_review_then_is_lgtm_true_when_artifact_lgtm_and_workspace_whitespace_only() {
    let t = tempfile::tempdir().unwrap();
    let workspace = t.path().join("review.md");
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&workspace, "  \n\t\n").unwrap();
    std::fs::write(&artifact, "LGTM\n").unwrap();
    assert!(sync_review_then_is_lgtm(&workspace, &artifact).unwrap());
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
fn tidy_reviewer_turn_double_clear_prevents_stale_lgtm_on_sync_attempt() {
    let (_t, artifact, workspace) = tmp_lgtm_artifact_and_workspace();
    clear_review_file(&artifact).unwrap();
    clear_review_file(&workspace).unwrap();
    let synced = super::sync_review_file_for_attempt(&artifact, &workspace).expect("sync");
    assert!(
        !synced.as_deref().is_some_and(is_lgtm_str),
        "stale LGTM must not survive the same double-clear prelude as run_tidy_review_attempt"
    );
}
