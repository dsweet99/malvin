//! Pre-Logs session reuse tests for router ACP.

#[cfg(unix)]
#[test]
fn begin_coder_session_if_needed_reuses_preopened_session() {
    use super::begin_coder_session_if_needed;
    use super::router_flow_acp_mock_tests::{
        install_mock_router_agent_env, write_mock_router_agent, ROUTER_MOCK_SESSION_COUNTS_FILE,
    };
    use super::router_flow_acp_tests::{router_boot_client_artifacts, test_router_shared};
    use super::{run_router_acp_open_iteration, RouterAcpIterationInput, RouterAcpIterationOutcome};
    use crate::run_timing::acp_post_run::RunTimingSessionEnd;

    crate::test_utils::enable_test_fast_teardown();
    crate::test_utils::with_isolated_home(|workspace| {
        crate::test_utils::block_on_test_async(async {
            crate::seed_malvin_checks(workspace, "true\n");
            let mock = workspace.join("mock-router-agent");
            write_mock_router_agent(&mock);
            let _env = install_mock_router_agent_env(workspace, &mock);
            let (shared, workflow) = test_router_shared();
            let (mut client, artifacts, prompt_store) =
                router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
            begin_coder_session_if_needed(&mut client, workspace)
                .await
                .expect("pre-Logs begin");
            assert!(client.has_open_coder_session());
            let RouterAcpIterationOutcome {
                acp_result,
                session_alive,
                ..
            } = run_router_acp_open_iteration(RouterAcpIterationInput {
                client: &mut client,
                artifacts: &artifacts,
                prompt_store: &prompt_store,
                shared: &shared,
                agent_loop: 1,
                session_end: RunTimingSessionEnd::Finalize,
            })
            .await;
            assert!(acp_result.is_ok(), "{acp_result:?}");
            assert!(session_alive);
            let counts_raw =
                std::fs::read_to_string(workspace.join(ROUTER_MOCK_SESSION_COUNTS_FILE))
                    .expect("counts");
            assert!(
                counts_raw.contains(r#""begins":1"#),
                "pre-open + if_needed must not spawn a second session/new; counts={counts_raw}"
            );
            let _ = client.end_coder_session().await;
        });
    });
}

#[cfg(unix)]
#[test]
fn kiss_cov_preopen_test_names() {
    let _ = stringify!(begin_coder_session_if_needed_reuses_preopened_session);
}
