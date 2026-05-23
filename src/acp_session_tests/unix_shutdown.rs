#[cfg(unix)]
mod shutdown_kills_descendants {
    use super::super::unix_helpers::{
        process_exists, wait_for_pid_file, write_descendant_spawning_acp_mock,
    };

    fn skip_without_writable_cgroups() -> bool {
        #[cfg(all(unix, target_os = "linux"))]
        {
            if !crate::acp_memory_containment::test_support::writable_cgroups_on_host() {
                eprintln!("SKIP shutdown_kills_agent_spawned_descendants: no writable cgroups");
                return true;
            }
        }
        false
    }

    async fn spawn_descendant_mock_session(
        tmp: &tempfile::TempDir,
        bin: &std::path::Path,
    ) -> (crate::acp::AcpSession, std::path::PathBuf) {
        let session = crate::acp::AcpSession::spawn(
            crate::acp::spawn_test_args::george_mock_spawn_args(tmp.path(), bin),
        )
        .await
        .expect("mock acp session should start");
        let prompt_log = tmp.path().join("prompt.log");
        (session, prompt_log)
    }

    async fn assert_descendant_killed_after_shutdown(session: crate::acp::AcpSession, pid: u32) {
        #[cfg(all(unix, target_os = "linux"))]
        let cgroup_leaf = session
            .memory_containment_cgroup_leaf_snapshot_for_tests()
            .expect("linux session expects memory containment cgroup leaf while active");

        session.shutdown().await.expect("shutdown should complete");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        #[cfg(all(unix, target_os = "linux"))]
        assert!(
            !cgroup_leaf.exists(),
            "cgroup leaf should be removed after session shutdown",
        );

        if process_exists(pid) {
            let _ = std::process::Command::new("kill")
                .arg("-KILL")
                .arg(pid.to_string())
                .status();
            panic!("shutdown left agent-spawned descendant process {pid} alive");
        }
    }

    #[tokio::test]
    async fn shutdown_kills_agent_spawned_descendants() {
        if skip_without_writable_cgroups() {
            return;
        }

        let tmp = tempfile::tempdir().unwrap();
        let bin = tmp.path().join("descendant-spawning-agent");
        write_descendant_spawning_acp_mock(&bin).await;

        let (session, prompt_log) = spawn_descendant_mock_session(&tmp, &bin).await;
        session
            .prompt("spawn descendant", &prompt_log, "test", None)
            .await
            .expect("mock prompt should complete");

        let pid = wait_for_pid_file(&tmp.path().join("descendant.pid")).await;
        assert!(
            process_exists(pid),
            "descendant should be alive before shutdown"
        );

        assert_descendant_killed_after_shutdown(session, pid).await;
    }
}
