use crate::agent_backend::{
    agent_backend_attach_run_timing_for_session, agent_backend_set_implement_display_name,
    agent_backend_set_run_timing, AgentBackend,
};
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::cli::SharedOpts;
use crate::prompts::PromptStore;
use crate::router_flow::router_flow_parse::router_wants_continue;
use crate::router_flow::router_flow_prompt;
use crate::run_timing::acp_post_run::RunTimingSessionEnd;

#[path = "router_flow_acp_support.rs"]
mod router_flow_acp_support;

#[path = "router_flow_post.rs"]
mod router_flow_post;

#[path = "router_flow_coder_prompts.rs"]
mod router_flow_coder_prompts;

use router_flow_acp_support::{
    abort_router_acp_session, end_router_acp_session, router_iteration_log_path,
    run_router_turns, snapshot_iteration_backups, RouterAcpSessionCtx,
};

pub(crate) struct RouterAcpIterationOutcome {
    pub acp_result: Result<(), String>,
    pub wants_continue: bool,
    pub iteration_backups: SessionDotfileBackups,
}

pub(crate) struct RouterAcpIterationInput<'a> {
    pub client: &'a mut AgentBackend,
    pub artifacts: &'a RunArtifacts,
    pub coder: &'a router_flow_prompt::RouterCoderRun,
    pub prompt_store: &'a PromptStore,
    pub shared: &'a SharedOpts,
    pub agent_loop: usize,
    pub session_end: RunTimingSessionEnd,
}

pub(crate) async fn run_router_acp_iteration(
    input: RouterAcpIterationInput<'_>,
) -> RouterAcpIterationOutcome {
    let RouterAcpIterationInput {
        client,
        artifacts,
        coder,
        prompt_store,
        shared,
        agent_loop,
        session_end,
    } = input;
    let work_dir = artifacts.work_dir.as_path();
    let log_path = router_iteration_log_path(artifacts, agent_loop);
    let timing = agent_backend_attach_run_timing_for_session(client);
    if let Err(e) = client.begin_coder_session(work_dir).await {
        agent_backend_set_run_timing(client, None);
        return RouterAcpIterationOutcome {
            acp_result: Err(e.to_string()),
            wants_continue: false,
            iteration_backups: snapshot_iteration_backups(work_dir),
        };
    }
    agent_backend_set_implement_display_name(client, "router");

    let mut session_ctx = RouterAcpSessionCtx {
        client,
        artifacts,
        coder,
        prompt_store,
        shared,
        log_path: log_path.as_path(),
        timing: &timing,
        session_end,
    };
    match run_router_turns(&mut session_ctx).await {
        Ok(turns) => {
            let agent_wants_continue = session_ctx
                .client
                .last_coder_prompt_agent_response()
                .as_deref()
                .is_some_and(router_wants_continue);
            let wants_continue = agent_wants_continue || turns.gate_wants_continue;
            let acp_result = end_router_acp_session(&mut session_ctx, Ok(())).await;
            RouterAcpIterationOutcome {
                acp_result,
                wants_continue,
                iteration_backups: turns.iteration_backups,
            }
        }
        Err(e) => RouterAcpIterationOutcome {
            acp_result: abort_router_acp_session(&mut session_ctx, e).await,
            wants_continue: false,
            iteration_backups: snapshot_iteration_backups(work_dir),
        },
    }
}

#[cfg(test)]
mod kiss_static_fn_item_refs {
    use super::{run_router_acp_iteration, RouterAcpIterationInput, RouterAcpIterationOutcome};

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = run_router_acp_iteration;
        let _ = super::router_flow_acp_support::run_router_turns;
        let _ = super::router_flow_coder_prompts::run_router_a_1_coder_prompt;
        let _ = super::router_flow_post::maybe_run_router_post_c_gates;
        let _ = std::any::type_name::<super::router_flow_post::RouterTurnsOutcome>();
        let _ = super::router_flow_acp_support::iteration_backups_after_router_a;
        let _ = std::any::type_name::<super::router_flow_acp_support::RouterAInitSnapshotInput>();
        let _ = super::router_flow_acp_support::workspace_has_valid_checks;
        let _: Option<RouterAcpIterationInput> = None;
        let _: Option<RouterAcpIterationOutcome> = None;
        let _: Option<super::router_flow_acp_support::RouterAcpSessionCtx<'_>> = None;
    }
}

#[cfg(test)]
#[path = "router_flow_acp_kiss_cov_tests.rs"]
mod router_flow_acp_kiss_cov_tests;

#[cfg(test)]
#[path = "router_flow_acp_mock_tests.rs"]
pub(crate) mod router_flow_acp_mock_tests;

#[cfg(test)]
#[path = "router_flow_acp_tests.rs"]
pub(crate) mod router_flow_acp_tests;
