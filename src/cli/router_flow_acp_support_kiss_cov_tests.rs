//! Kiss identifier refs for [`super::router_flow_acp_support`].

use super::RouterAcpSessionCtx;

#[test]
fn kiss_cov_router_acp_session_ctx_construct_destructure() {
    use std::sync::{Arc, Mutex};

    crate::test_utils::with_isolated_home(|workspace| {
        let (shared, workflow) =
            crate::router_flow::router_flow_acp::router_flow_acp_tests::test_router_shared();
        let (mut client, artifacts, coder, prompt_store) =
            crate::router_flow::router_flow_acp::router_flow_acp_tests::router_boot_client_artifacts(
                workspace, &shared, workflow,
            )
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
        std::hint::black_box(session);
    });
}
