//! Tests for [`super`] agent loop driver (happy path / done skip).

#[cfg(unix)]
mod unix_cov {
    use super::super::{run_router_agent_loops, RouterAgentLoopInput, RouterAgentLoopOutcome};
    use crate::router_flow::router_flow_acp::router_flow_acp_mock_tests::{
        install_mock_router_agent_env, install_mock_router_agent_env_with_script,
    };
    use crate::router_flow::router_flow_acp::router_flow_acp_tests::{
        router_boot_client_artifacts, test_router_shared,
    };

    #[test]
    fn run_router_agent_loops_single_session_a_to_b() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-agent");
                let _env = install_mock_router_agent_env(workspace, &mock);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, prompt_store) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAgentLoopOutcome { last_acp, .. } = run_router_agent_loops(
                    RouterAgentLoopInput {
                        client: &mut client,
                        artifacts: &artifacts,
                        prompt_store: &prompt_store,
                        shared: &shared,
                        max_loops: 1,
                    },
                )
                .await
                .expect("loops");
                last_acp.expect("acp");
                assert!(artifacts.log_path("router_1").is_file());
                assert!(!artifacts.log_path("router_2").is_file());
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
                    "single begin around header → kpop → a → b → summarize: {counts_raw}"
                );
                assert_eq!(
                    counts.get("prompts").and_then(serde_json::Value::as_u64),
                    Some(5),
                    "one session serves five prompts: {counts_raw}"
                );
                assert!(
                    workspace.join(".malvin_router_mock_saw_summarize").is_file(),
                    "summarize prompt body must reach the open coder session before teardown"
                );
            });
        });
    }

    #[test]
    fn run_router_agent_loops_skips_b_when_done() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-no-work");
                crate::router_flow::router_flow_acp::router_flow_acp_mock_no_work_tests::write_mock_router_agent_all_no_work(
                    &mock,
                );
                let _env = install_mock_router_agent_env_with_script(workspace, &mock);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, prompt_store) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAgentLoopOutcome { last_acp, .. } = run_router_agent_loops(
                    RouterAgentLoopInput {
                        client: &mut client,
                        artifacts: &artifacts,
                        prompt_store: &prompt_store,
                        shared: &shared,
                        max_loops: 1,
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
                    "{counts_raw}"
                );
                assert_eq!(
                    counts.get("prompts").and_then(serde_json::Value::as_u64),
                    Some(4),
                    "header + kpop_common + router_a + summarize, no b: {counts_raw}"
                );
            });
        });
    }

    #[test]
    fn run_router_agent_loops_second_begin_after_work_then_done() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-multi");
                crate::router_flow::router_flow_acp::router_flow_acp_mock_no_work_tests::write_mock_router_agent_work_then_no_work(
                    &mock,
                );
                let _env = install_mock_router_agent_env_with_script(workspace, &mock);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, prompt_store) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                run_router_agent_loops(RouterAgentLoopInput {
                    client: &mut client,
                    artifacts: &artifacts,
                    prompt_store: &prompt_store,
                    shared: &shared,
                    max_loops: 2,
                })
                .await
                .expect("loops")
                .last_acp
                .expect("acp");
                assert!(artifacts.log_path("router_1").is_file() && artifacts.log_path("router_2").is_file());
                let counts: serde_json::Value = serde_json::from_str(
                    &std::fs::read_to_string(workspace.join(
                        crate::router_flow::router_flow_acp::router_flow_acp_mock_tests::ROUTER_MOCK_SESSION_COUNTS_FILE,
                    ))
                    .expect("counts"),
                )
                .expect("parse");
                assert_eq!(counts["begins"], 2);
                // S1: header+kpop+a+b=4; S2: header+kpop+a+summarize=4 → 8
                assert_eq!(counts["prompts"], 8);
                assert_eq!(
                    std::fs::read_to_string(workspace.join(".malvin_router_mock_summarize_count"))
                        .unwrap_or_else(|_| "0".into())
                        .trim(),
                    "1"
                );
            });
        });
    }
}
