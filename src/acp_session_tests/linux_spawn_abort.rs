#[cfg(all(unix, target_os = "linux"))]
mod linux_cgroup_verify_abort {
    #[tokio::test]
    async fn acp_session_spawn_aborts_when_linux_cgroup_verify_fails() {
        if !crate::acp_memory_containment::test_support::writable_cgroups_on_host() {
            eprintln!(
                "SKIP acp_session_spawn_aborts_when_linux_cgroup_verify_fails: no writable cgroups"
            );
            return;
        }

        let tmp = tempfile::tempdir().unwrap();
        match crate::acp::AcpSession::spawn(
            crate::acp::spawn_test_args::george_mock_spawn_args(
                tmp.path(),
                std::path::Path::new("/bin/false"),
            ),
        )
        .await
        {
            Ok(_) => panic!("spawn should fail when linux cgroup verify cannot succeed"),
            Err(err) => assert!(
                err.contains("verify failed"),
                "expected verify failure in error message, got: {err}"
            ),
        }
    }
}
