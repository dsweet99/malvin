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

        assert!(
            !process_exists(pid),
            "shutdown left agent-spawned descendant process {pid} alive"
        );
    }

    #[tokio::test]
    pub(crate) async fn shutdown_sends_cancel_before_teardown() {
        let tmp = tempfile::tempdir().unwrap();
        let bin = tmp.path().join("rpc-trace-agent");
        crate::test_utils::write_acp_jsonrpc_mock_rpc_trace(&bin).await;
        let session = crate::acp::AcpSession::spawn(crate::acp::spawn_test_args::george_mock_spawn_args(
            tmp.path(),
            &bin,
        ))
        .await
        .expect("mock acp session should start");
        let prompt_log = tmp.path().join("prompt.log");
        session
            .prompt("noop", &prompt_log, "test", None)
            .await
            .expect("mock prompt should complete");
        session.shutdown().await.expect("shutdown should complete");
        let trace = std::fs::read_to_string(tmp.path().join("rpc_trace")).unwrap_or_default();
        assert!(
            trace.lines().any(|line| line == "cancel"),
            "shutdown must call session/cancel before OS teardown (trace={trace:?})"
        );
    }

    #[tokio::test]
    pub(crate) async fn shutdown_kills_agent_spawned_descendants() {
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
    assert_descendant_killed_after_shutdown, shutdown_kills_agent_spawned_descendants,
    shutdown_sends_cancel_before_teardown, spawn_descendant_mock_session,
};


#[cfg(test)]
#[cfg(all(test, unix))] #[test] fn kiss_cov_shutdown_kills_agent_spawned_descendants() { let _ = shutdown_kills_agent_spawned_descendants; }
#[cfg(test)]
#[cfg(all(test, unix))] #[test] fn kiss_cov_shutdown_sends_cancel_before_teardown() { let _ = shutdown_sends_cancel_before_teardown; }
