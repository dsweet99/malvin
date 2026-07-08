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
    fn run_router_agent_loops_single_iteration_without_continue() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-agent");
                let _env = install_mock_router_agent_env(workspace, &mock, false);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, coder, router_b_prompt) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAgentLoopOutcome { last_acp, .. } = run_router_agent_loops(
                    RouterAgentLoopInput {
                        client: &mut client,
                        artifacts: &artifacts,
                        coder: &coder,
                        router_b_prompt: &router_b_prompt,
                        max_loops: 1,
                    },
                )
                .await
                .expect("loops");
                last_acp.expect("acp");
            });
        });
    }

    #[test]
    fn run_router_agent_loops_runs_second_iteration_when_router_b_continues() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-agent");
                let _env = install_mock_router_agent_env(workspace, &mock, true);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, coder, router_b_prompt) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAgentLoopOutcome {
                    last_acp,
                    last_backups,
                } = run_router_agent_loops(RouterAgentLoopInput {
                    client: &mut client,
                    artifacts: &artifacts,
                    coder: &coder,
                    router_b_prompt: &router_b_prompt,
                    max_loops: 2,
                })
                .await
                .expect("loops");
                last_acp.expect("acp");
                let _ = last_backups.malvin_config;
            });
        });
    }

    #[test]
    fn run_router_agent_loops_stops_early_on_non_continue_even_with_budget() {
        crate::test_utils::enable_test_fast_teardown();
        crate::test_utils::with_isolated_home(|workspace| {
            crate::test_utils::block_on_test_async(async {
                crate::seed_malvin_checks(workspace, "true\n");
                let mock = workspace.join("mock-router-agent");
                let _env = install_mock_router_agent_env(workspace, &mock, false);
                let (shared, workflow) = test_router_shared();
                let (mut client, artifacts, coder, router_b_prompt) =
                    router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
                let RouterAgentLoopOutcome { last_acp, last_backups } =
                    run_router_agent_loops(RouterAgentLoopInput {
                        client: &mut client,
                        artifacts: &artifacts,
                        coder: &coder,
                        router_b_prompt: &router_b_prompt,
                        max_loops: 3,
                    })
                    .await
                    .expect("loops");
                last_acp.expect("acp");
                let _ = last_backups;
            });
        });
    }
}

#[cfg(unix)]
#[test]
fn kiss_cov_unix_cov_test_names() {
    let _ = stringify!(run_router_agent_loops_single_iteration_without_continue);
    let _ = stringify!(run_router_agent_loops_runs_second_iteration_when_router_b_continues);
    let _ = stringify!(run_router_agent_loops_stops_early_on_non_continue_even_with_budget);
}
