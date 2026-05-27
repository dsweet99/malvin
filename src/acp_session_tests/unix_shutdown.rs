#[cfg(unix)]
pub(crate) mod shutdown_kills_descendants {
    use super::super::unix_helpers::{
        process_exists, wait_for_pid_file, write_descendant_spawning_acp_mock,
    };

    pub(crate) async fn spawn_descendant_mock_session(
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

    pub(crate) async fn assert_descendant_killed_after_shutdown(
        session: crate::acp::AcpSession,
        pid: u32,
    ) {
        session.shutdown().await.expect("shutdown should complete");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

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

#[cfg(all(test, unix))]
pub(crate) use shutdown_kills_descendants::{
    assert_descendant_killed_after_shutdown, spawn_descendant_mock_session,
};
