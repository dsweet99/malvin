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
fn capture_records_both_axes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let checkpoint = ForkState::capture(tmp.path(), 7);
    assert_eq!(checkpoint.message_checkpoint_len, 7);
    assert_eq!(
        checkpoint.workspace_manifest_hash,
        workspace_manifest_hash(tmp.path())
    );
}

#[test]
fn is_diverged_when_transcript_grows() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let checkpoint = ForkState::capture(tmp.path(), 3);
    let hash = workspace_manifest_hash(tmp.path());
    assert!(checkpoint.is_diverged(4, &hash));
    assert!(!checkpoint.transcript_matches(4));
    assert!(checkpoint.workspace_matches(&hash));
}

#[test]
fn is_diverged_when_workspace_hash_changes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let checkpoint = ForkState::capture(tmp.path(), 5);
    assert!(checkpoint.is_diverged(5, "other:deadbeef"));
    assert!(checkpoint.transcript_matches(5));
    assert!(!checkpoint.workspace_matches("other:deadbeef"));
}

#[test]
fn is_not_diverged_when_both_axes_match() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let checkpoint = ForkState::capture(tmp.path(), 2);
    let hash = workspace_manifest_hash(tmp.path());
    assert!(!checkpoint.is_diverged(2, &hash));
    assert!(checkpoint.transcript_matches(2));
    assert!(checkpoint.workspace_matches(&hash));
}

#[test]
fn from_tuple_round_trip() {
    let state = ForkState {
        message_checkpoint_len: 9,
        workspace_manifest_hash: "git:abc".into(),
    };
    let (len, hash): (usize, String) = state.clone().into();
    assert_eq!(len, 9);
    assert_eq!(hash, "git:abc");
    let back = ForkState::from((len, hash));
    assert_eq!(back, state);
}
