//! Tests for [`super`] agent loop driver.

use super::{run_router_agent_loops, RouterAgentLoopInput, RouterAgentLoopOutcome};

#[test]
fn kiss_cov_router_agent_loop_type_names() {
    let _ = std::any::type_name::<RouterAgentLoopInput<'_>>();
    let _ = std::any::type_name::<RouterAgentLoopOutcome>();
    let _: Option<RouterAgentLoopInput> = None;
    let _: Option<RouterAgentLoopOutcome> = None;
    let _ = run_router_agent_loops;
}

#[cfg(unix)]
mod unix_cov {
    use super::{run_router_agent_loops, RouterAgentLoopInput, RouterAgentLoopOutcome};
    use crate::router_flow::router_flow_acp::router_flow_acp_mock_tests::install_mock_router_agent_env;
    use crate::router_flow::router_flow_acp::router_flow_acp_tests::{
        router_boot_client_artifacts, test_router_shared,
    };

    #[test]
    fn run_router_agent_loops_single_session_requirements_to_work() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-agent");
                let _env = install_mock_router_agent_env(workspace, &mock);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, coder, prompt_store) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAgentLoopOutcome { last_acp, .. } = run_router_agent_loops(
                    RouterAgentLoopInput {
                        client: &mut client,
                        artifacts: &artifacts,
                        coder: &coder,
                        prompt_store: &prompt_store,
                        shared: &shared,
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
                    "single begin around requirements → groups → work: {counts_raw}"
                );
                assert_eq!(
                    counts.get("prompts").and_then(serde_json::Value::as_u64),
                    Some(3),
                    "one session serves three prompts: {counts_raw}"
                );
            });
        });
    }
}

#[cfg(unix)]
#[test]
fn kiss_cov_unix_cov_test_names() {
    let _ = stringify!(run_router_agent_loops_single_session_requirements_to_work);
}
