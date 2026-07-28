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
                let counts: serde_json::Value = serde_json::from_str(
                    &std::fs::read_to_string(workspace.join(
                        crate::router_flow::router_flow_acp::router_flow_acp_mock_tests::ROUTER_MOCK_SESSION_COUNTS_FILE,
                    ))
                    .expect("counts"),
                )
                .expect("parse");
                assert_eq!(counts["begins"], 2);
                assert_eq!(counts["prompts"], 5);
                assert_eq!(
                    std::fs::read_to_string(workspace.join(".malvin_router_mock_summarize_count"))
                        .unwrap_or_else(|_| "0".into())
                        .trim(),
                    "1"
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
                let counts: serde_json::Value = serde_json::from_str(
                    &std::fs::read_to_string(workspace.join(
                        crate::router_flow::router_flow_acp::router_flow_acp_mock_tests::ROUTER_MOCK_SESSION_COUNTS_FILE,
                    ))
                    .expect("counts"),
                )
                .expect("parse");
                assert_eq!(counts["begins"], 1);
            });
        });
    }
}
