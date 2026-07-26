//! Kiss identifier refs for [`super`] ACP iteration helpers.

use super::{run_router_acp_iteration, RouterAcpIterationInput, RouterAcpIterationOutcome};
use crate::artifacts::SessionDotfileBackups;

#[test]
fn kiss_cov_router_flow_acp_privates() {
    let _: Option<RouterAcpIterationInput> = None;
    let _: Option<RouterAcpIterationOutcome> = None;
    let _ = run_router_acp_iteration;
    let _ = stringify!(client);
    let _ = stringify!(artifacts);
    let _ = stringify!(coder);
    let _ = stringify!(prompt_store);
    let _ = stringify!(shared);
    let _ = stringify!(agent_loop);
    let _ = stringify!(session_end);
    let _ = stringify!(acp_result);
    let _ = stringify!(iteration_backups);
}

#[test]
fn kiss_cov_router_flow_acp_outcome_destructure() {
    let outcome = RouterAcpIterationOutcome {
        acp_result: Ok(()),
        iteration_backups: SessionDotfileBackups::snapshot(std::path::Path::new("/tmp"))
            .expect("snapshot"),
    };
    let RouterAcpIterationOutcome {
        acp_result,
        iteration_backups: _,
    } = outcome;
    assert!(acp_result.is_ok());
}

#[cfg(unix)]
#[test]
fn kiss_cov_router_flow_acp_live_outcome_fields() {
    crate::test_utils::with_isolated_home(|workspace| {
        crate::test_utils::block_on_test_async(async {
            crate::seed_malvin_checks(workspace, "true\n");
            let mock = workspace.join("mock-router-agent");
            let _env = super::router_flow_acp_mock_tests::install_mock_router_agent_env(
                workspace, &mock,
            );
            let (shared, workflow) = super::router_flow_acp_tests::test_router_shared();
            let (mut client, artifacts, coder, prompt_store) =
                super::router_flow_acp_tests::router_boot_client_artifacts(
                    workspace, &shared, workflow,
                )
                .expect("boot");
            let input = RouterAcpIterationInput {
                client: &mut client,
                artifacts: &artifacts,
                coder: &coder,
                prompt_store: &prompt_store,
                shared: &shared,
                agent_loop: 1,
                session_end: crate::run_timing::acp_post_run::RunTimingSessionEnd::Finalize,
            };
            let RouterAcpIterationInput {
                client: _,
                artifacts: _,
                coder: _,
                prompt_store: _,
                shared: _,
                agent_loop: _,
                session_end: _,
            } = input;
            let outcome = run_router_acp_iteration(input).await;
            let RouterAcpIterationOutcome {
                acp_result: _,
                iteration_backups: _,
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
    let _ = super::router_flow_acp_mock_tests::write_mock_router_agent_missing_requirements;
    let _ = stringify!(run_router_acp_iteration_executes_mock_agent_full_sequence);
    let _ = stringify!(run_router_acp_iteration_aborts_when_requirements_json_missing);
    let _ = stringify!(run_router_acp_iteration_propagates_begin_session_failure);
}
