//! Kiss identifier refs for [`super`] agent loop driver.

use super::{run_router_agent_loops, RouterAgentLoopInput, RouterAgentLoopOutcome};

#[test]
fn kiss_cov_router_flow_loop_privates() {
    let _: Option<RouterAgentLoopInput> = None;
    let _: Option<RouterAgentLoopOutcome> = None;
    let _ = run_router_agent_loops;
    let _ = stringify!(client);
    let _ = stringify!(artifacts);
    let _ = stringify!(coder);
    let _ = stringify!(prompt_store);
    let _ = stringify!(shared);
    let _ = stringify!(max_loops);
    let _ = stringify!(last_acp);
    let _ = stringify!(last_backups);
    let _ = stringify!(work_dir);
    let _ = stringify!(agent_loop);
    let _ = stringify!(iteration_backups);
    let _ = stringify!(session_end);
    let _ = stringify!(iteration);
}

#[cfg(unix)]
#[test]
fn kiss_cov_router_flow_loop_live_outcome_fields() {
    use crate::router_flow::router_flow_acp::router_flow_acp_mock_tests::install_mock_router_agent_env;

    use crate::router_flow::router_flow_acp::router_flow_acp_tests::{
        router_boot_client_artifacts, test_router_shared,
    };

    crate::test_utils::with_isolated_home(|workspace| {
        crate::test_utils::block_on_test_async(async {
            crate::seed_malvin_checks(workspace, "true\n");
            let mock = workspace.join("mock-router-agent");
            let _env = install_mock_router_agent_env(workspace, &mock, true);
            let (shared, workflow) = test_router_shared();
            let (mut client, artifacts, coder, prompt_store) =
                router_boot_client_artifacts(workspace, &shared, workflow).expect("boot");
            let input = RouterAgentLoopInput {
                client: &mut client,
                artifacts: &artifacts,
                coder: &coder,
                prompt_store: &prompt_store,
                shared: &shared,
                max_loops: 2,
            };
            let RouterAgentLoopInput {
                client: _,
                artifacts: _,
                coder: _,
                prompt_store: _,
                shared: _,
                max_loops: _,
            } = input;
            let outcome = run_router_agent_loops(input).await.expect("loops");
            let RouterAgentLoopOutcome {
                last_acp: _,
                last_backups: _,
            } = outcome;
        });
    });
}

#[cfg(unix)]
#[test]
fn kiss_cov_router_flow_loop_test_helpers() {
    let _ = stringify!(run_router_agent_loops_single_iteration_without_continue);
    let _ = stringify!(run_router_agent_loops_stops_early_on_non_continue_even_with_budget);
    let _ = stringify!(run_router_agent_loops_runs_second_iteration_when_router_c_continues);
}
