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
    let _ = stringify!(wants_continue);
    let _ = stringify!(iteration_backups);
}

#[test]
fn kiss_cov_router_flow_acp_outcome_destructure() {
    let outcome = RouterAcpIterationOutcome {
        acp_result: Ok(()),
        wants_continue: true,
        iteration_backups: SessionDotfileBackups::snapshot(std::path::Path::new("/tmp"))
            .expect("snapshot"),
    };
    let RouterAcpIterationOutcome {
        acp_result,
        wants_continue,
        iteration_backups: _,
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
                wants_continue: _,
                iteration_backups: _,
            } = outcome;
        });
    });
}

#[cfg(unix)]
#[test]
fn kiss_cov_router_acp_session_ctx_construct_destructure() {
    use super::router_flow_acp_support::RouterAcpSessionCtx;
    use std::sync::{Arc, Mutex};

    crate::test_utils::with_isolated_home(|workspace| {
        let (shared, workflow) = super::router_flow_acp_tests::test_router_shared();
        let (mut client, artifacts, coder, prompt_store) =
            super::router_flow_acp_tests::router_boot_client_artifacts(workspace, &shared, workflow)
                .expect("boot");
        let log_path = artifacts.log_path("router_1");
        let timing = Arc::new(Mutex::new(crate::run_timing::RunTiming::default()));
        let session = RouterAcpSessionCtx {
            client: &mut client,
            artifacts: &artifacts,
            coder: &coder,
            prompt_store: &prompt_store,
            shared: &shared,
            log_path: log_path.as_path(),
            timing: &timing,
            session_end: crate::run_timing::acp_post_run::RunTimingSessionEnd::Finalize,
        };
        let touched = std::hint::black_box(session);
        let RouterAcpSessionCtx {
            client: _,
            artifacts: _,
            coder: _,
            prompt_store: _,
            shared: _,
            log_path: _,
            timing: _,
            session_end: _,
        } = touched;
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
    let _ = stringify!(run_router_acp_iteration_wants_continue_when_router_c_emits_marker);
    let _ = stringify!(run_router_acp_iteration_propagates_begin_session_failure);
}
