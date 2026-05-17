use super::{is_lgtm_str, read_artifact_review_for_fanout_attempt};

#[test]
fn read_artifact_review_for_fanout_attempt_ignores_workspace_lgtm_when_artifact_empty() {
    let t = tempfile::tempdir().unwrap();
    let workspace = t.path().join("review.md");
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&workspace, "LGTM\n").unwrap();
    let out = read_artifact_review_for_fanout_attempt(&artifact).unwrap();
    assert_eq!(out, None);
    assert!(
        !artifact.exists()
            || std::fs::read_to_string(&artifact)
                .unwrap()
                .trim()
                .is_empty(),
        "fan-out read must not promote workspace LGTM into artifact"
    );
}

#[test]
fn read_artifact_review_for_fanout_attempt_returns_nonempty_artifact_text() {
    let t = tempfile::tempdir().unwrap();
    let artifact = t.path().join("run").join("review.md");
    std::fs::create_dir_all(artifact.parent().unwrap()).unwrap();
    std::fs::write(&artifact, "  LGTM\n").unwrap();
    let out = read_artifact_review_for_fanout_attempt(&artifact)
        .unwrap()
        .expect("non-empty artifact");
    assert!(is_lgtm_str(&out));
}
