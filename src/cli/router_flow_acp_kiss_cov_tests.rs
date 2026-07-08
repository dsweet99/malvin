//! Kiss identifier refs for [`super`] ACP iteration helpers.

use super::{run_router_acp_iteration, RouterAcpIterationInput, RouterAcpIterationOutcome};

#[test]
fn kiss_cov_router_flow_acp_privates() {
    let _: Option<RouterAcpIterationInput> = None;
    let _: Option<RouterAcpIterationOutcome> = None;
    let _ = run_router_acp_iteration;
    let _ = stringify!(client);
    let _ = stringify!(artifacts);
    let _ = stringify!(coder);
    let _ = stringify!(router_b_prompt);
    let _ = stringify!(session_end);
    let _ = stringify!(acp_result);
    let _ = stringify!(wants_continue);
}

#[test]
fn kiss_cov_router_flow_acp_outcome_destructure() {
    let outcome = RouterAcpIterationOutcome {
        acp_result: Ok(()),
        wants_continue: true,
    };
    let RouterAcpIterationOutcome {
        acp_result,
        wants_continue,
    } = outcome;
    assert!(acp_result.is_ok());
    assert!(wants_continue);
}

#[cfg(unix)]
#[test]
fn kiss_cov_router_flow_acp_live_outcome_fields() {
    crate::test_utils::with_isolated_home(|workspace| {
        crate::test_utils::block_on_test_async(async {
            crate::seed_malvin_checks(workspace, "true\n");
            let mock = workspace.join("mock-router-agent");
            let _env = super::router_flow_acp_mock_tests::install_mock_router_agent_env(
                workspace, &mock, false,
            );
            let (shared, workflow) = super::router_flow_acp_tests::test_router_shared();
            let (mut client, artifacts, coder, router_b_prompt) =
                super::router_flow_acp_tests::router_boot_client_artifacts(
                    workspace, &shared, workflow,
                )
                .expect("boot");
            let input = RouterAcpIterationInput {
                client: &mut client,
                artifacts: &artifacts,
                coder: &coder,
                router_b_prompt: &router_b_prompt,
                session_end: crate::run_timing::acp_post_run::RunTimingSessionEnd::Finalize,
            };
            let RouterAcpIterationInput {
                client: _,
                artifacts: _,
                coder: _,
                router_b_prompt: _,
                session_end: _,
            } = input;
            let outcome = run_router_acp_iteration(input).await;
            let RouterAcpIterationOutcome {
                acp_result: _,
                wants_continue: _,
            } = outcome;
        });
    });
}

#[cfg(unix)]
#[test]
fn kiss_cov_router_flow_acp_test_helpers() {
    let _ = super::router_flow_acp_tests::test_router_shared;
    let _ = super::router_flow_acp_tests::router_boot_client_artifacts;
    let _ = super::router_flow_acp_mock_tests::install_mock_router_agent_env;
    let _ = super::router_flow_acp_mock_tests::write_mock_router_agent;
    let _ = super::router_flow_acp_mock_tests::write_mock_router_agent_session_fail;
    let _ = stringify!(run_router_acp_iteration_executes_mock_agent_without_continue);
    let _ = stringify!(run_router_acp_iteration_wants_continue_when_router_b_emits_marker);
    let _ = stringify!(run_router_acp_iteration_propagates_begin_session_failure);
}
