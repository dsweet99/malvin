//! Tests for [`super`] agent loop driver with `--gates` restart behavior.

#[cfg(unix)]
mod unix_gates {
    use super::super::{run_router_agent_loops, RouterAgentLoopInput, RouterAgentLoopOutcome};
    use crate::router_flow::router_flow_acp::router_flow_acp_mock_tests::install_mock_router_agent_env_with_script;
    use crate::router_flow::router_flow_acp::router_flow_acp_tests::{
        router_boot_client_artifacts, test_router_shared,
    };

    #[test]
    fn run_router_agent_loops_gates_fail_restarts_even_on_all_no_work() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "false\n");
                let mock = workspace.join("mock-router-gates");
                crate::router_flow::router_flow_acp::router_flow_acp_mock_counting_tests::write_mock_router_agent_all_no_work_counting(
                    &mock,
                );
                let _env = install_mock_router_agent_env_with_script(workspace, &mock);
                let (mut shared, workflow) = test_router_shared();
                shared.gates = true;
                let (mut client, artifacts, coder, prompt_store) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let Err(err) = run_router_agent_loops(RouterAgentLoopInput {
                    client: &mut client,
                    artifacts: &artifacts,
                    coder: &coder,
                    prompt_store: &prompt_store,
                    shared: &shared,
                    max_loops: 2,
                })
                .await
                else {
                    panic!("exhausted gates should fail");
                };
                assert!(
                    err.contains("Workspace checks")
                        || err.contains("GATE_FAILURE")
                        || err.contains("malvin tidy"),
                    "gate failure message, got: {err}"
                );
                let counts_path = workspace.join(
                    crate::router_flow::router_flow_acp::router_flow_acp_mock_tests::ROUTER_MOCK_SESSION_COUNTS_FILE,
                );
                let counts_raw =
                    std::fs::read_to_string(&counts_path).expect("read mock session counts");
                let counts: serde_json::Value =
                    serde_json::from_str(&counts_raw).expect("parse mock session counts");
                assert_eq!(
                    counts.get("begins").and_then(serde_json::Value::as_u64),
                    Some(2),
                    "failing gates restart despite all_no_work: {counts_raw}"
                );
            });
        });
    }

    #[test]
    fn run_router_agent_loops_gates_pass_stops_after_one_begin() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-gates-ok");
                crate::router_flow::router_flow_acp::router_flow_acp_mock_counting_tests::write_mock_router_agent_all_no_work_counting(
                    &mock,
                );
                let _env = install_mock_router_agent_env_with_script(workspace, &mock);
                let (mut shared, workflow) = test_router_shared();
                shared.gates = true;
                let (mut client, artifacts, coder, prompt_store) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAgentLoopOutcome { last_acp, .. } = run_router_agent_loops(
                    RouterAgentLoopInput {
                        client: &mut client,
                        artifacts: &artifacts,
                        coder: &coder,
                        prompt_store: &prompt_store,
                        shared: &shared,
                        max_loops: 2,
                    },
                )
                .await
                .expect("loops");
                last_acp.expect("acp");
                let counts_path = workspace.join(
                    crate::router_flow::router_flow_acp::router_flow_acp_mock_tests::ROUTER_MOCK_SESSION_COUNTS_FILE,
                );
                let counts_raw =
                    std::fs::read_to_string(&counts_path).expect("read mock session counts");
                let counts: serde_json::Value =
                    serde_json::from_str(&counts_raw).expect("parse mock session counts");
                assert_eq!(
                    counts.get("begins").and_then(serde_json::Value::as_u64),
                    Some(1),
                    "passing gates stop early: {counts_raw}"
                );
            });
        });
    }
}
