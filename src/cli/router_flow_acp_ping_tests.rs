//! Router ACP iteration tests for Cursor HTTP/2 PING `RetriableError` handling.

#[cfg(unix)]
mod unix_cov {
    use super::super::router_flow_acp_mock_tests::install_mock_router_agent_env_with_script;
    use super::super::router_flow_acp_ping_mock_tests::{
        write_mock_router_agent_requirements_ping_then_ok,
        write_mock_router_agent_requirements_ping_timeout,
    };
    use super::super::router_flow_acp_tests::{
        router_boot_client_artifacts, test_router_shared,
    };
    use super::super::{run_router_acp_iteration, RouterAcpIterationInput, RouterAcpIterationOutcome};
    use crate::run_timing::acp_post_run::RunTimingSessionEnd;

    #[test]
    fn run_router_acp_iteration_reports_ping_timeout_not_missing_requirements() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-agent-ping");
                write_mock_router_agent_requirements_ping_timeout(&mock);
                let _env = install_mock_router_agent_env_with_script(workspace, &mock);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, coder, prompt_store) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAcpIterationOutcome { acp_result, .. } =
                    run_router_acp_iteration(RouterAcpIterationInput {
                        client: &mut client,
                        artifacts: &artifacts,
                        coder: &coder,
                        prompt_store: &prompt_store,
                        shared: &shared,
                        agent_loop: 1,
                        session_end: RunTimingSessionEnd::Finalize,
                    })
                    .await;
                let err = acp_result.expect_err("must fail on PING transport error");
                assert!(
                    err.contains("PING timed out"),
                    "primary error must be PING transport, got: {err}"
                );
                assert!(
                    !err.contains("missing or unreadable"),
                    "must not surface missing-file secondary, got: {err}"
                );
                assert!(!crate::artifacts::review_requirements_json(&artifacts).is_file());
            });
        });
    }

    #[test]
    fn run_router_acp_iteration_retries_requirements_after_ping_timeout() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-agent-ping-retry");
                write_mock_router_agent_requirements_ping_then_ok(&mock);
                let _env = install_mock_router_agent_env_with_script(workspace, &mock);
                let (mut shared, workflow) = test_router_shared();
                shared.max_acp_retries = 2;
                let (mut client, artifacts, coder, prompt_store) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAcpIterationOutcome { acp_result, .. } =
                    run_router_acp_iteration(RouterAcpIterationInput {
                        client: &mut client,
                        artifacts: &artifacts,
                        coder: &coder,
                        prompt_store: &prompt_store,
                        shared: &shared,
                        agent_loop: 1,
                        session_end: RunTimingSessionEnd::Finalize,
                    })
                    .await;
                acp_result.expect("requirements should succeed after PING retry");
                assert!(crate::artifacts::review_requirements_json(&artifacts).is_file());
            });
        });
    }
}

#[cfg(unix)]
#[test]
fn kiss_cov_ping_unix_cov_test_names() {
    let _ = stringify!(run_router_acp_iteration_reports_ping_timeout_not_missing_requirements);
    let _ = stringify!(run_router_acp_iteration_retries_requirements_after_ping_timeout);
}
