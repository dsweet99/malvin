#[tokio::test]
async fn dropping_cloned_acp_session_keeps_containment_active() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cgroup_dir = dir.path().join("malvin-acp-test-leaf");
    std::fs::create_dir(&cgroup_dir).expect("cgroup dir");
    let session =
        crate::acp::test_captive_session::captive_cat_acp_session_with_containment_for_tests(
            dir.path(),
            crate::acp_memory_containment::test_support::active_with_cgroup_dir(cgroup_dir.clone()),
        );
    let prompt_handle = session.clone();
    drop(prompt_handle);
    assert!(
        session.memory_containment_active_for_tests(),
        "dropping a cloned AcpSession must not tear down cgroup while the session lives"
    );
    drop(session);
    assert!(
        !cgroup_dir.exists(),
        "last AcpSession drop must remove containment cgroup"
    );
}
