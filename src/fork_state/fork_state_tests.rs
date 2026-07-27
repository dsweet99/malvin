use super::{ForkState, workspace_manifest_hash};

#[test]
fn workspace_manifest_hash_is_stable_for_same_dir() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let a = workspace_manifest_hash(tmp.path());
    let b = workspace_manifest_hash(tmp.path());
    assert_eq!(a, b);
    assert!(!a.is_empty());
}

#[test]
fn capture_records_memory_and_workspace() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let checkpoint = ForkState::capture(tmp.path(), "h1", "prev");
    assert_eq!(checkpoint.history, "h1");
    assert_eq!(checkpoint.previous_response, "prev");
    assert_eq!(
        checkpoint.workspace_manifest_hash,
        workspace_manifest_hash(tmp.path())
    );
}

#[test]
fn is_diverged_when_history_changes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let checkpoint = ForkState::capture(tmp.path(), "h", "p");
    let hash = workspace_manifest_hash(tmp.path());
    assert!(checkpoint.is_diverged("h2", "p", &hash));
    assert!(!checkpoint.memory_matches("h2", "p"));
    assert!(checkpoint.workspace_matches(&hash));
}

#[test]
fn is_diverged_when_workspace_hash_changes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let checkpoint = ForkState::capture(tmp.path(), "h", "p");
    assert!(checkpoint.is_diverged("h", "p", "other:deadbeef"));
    assert!(checkpoint.memory_matches("h", "p"));
    assert!(!checkpoint.workspace_matches("other:deadbeef"));
}

#[test]
fn is_not_diverged_when_both_axes_match() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let checkpoint = ForkState::capture(tmp.path(), "h", "p");
    let hash = workspace_manifest_hash(tmp.path());
    assert!(!checkpoint.is_diverged("h", "p", &hash));
}
