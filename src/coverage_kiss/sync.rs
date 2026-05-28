#[test]
fn smoke_review_sync() {
    assert!(crate::review_sync::is_lgtm_str("LGTM"));
    assert!(crate::review_sync::is_lgtm_str("\u{FEFF}LGTM"));

    let tmp = tempfile::tempdir().expect("tempdir");
    let art = tmp.path().join("artifact_review.md");
    assert!(
        crate::review_sync::read_artifact_review_for_fanout_attempt(&art)
            .expect("read")
            .is_none()
    );

    let synced = crate::review_sync::sync_review_file_for_attempt(&art).expect("sync");
    assert!(synced.is_none(), "missing artifact must not produce review text");
}
