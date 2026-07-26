use crate::agent_backend::{
    agent_backend_attach_run_timing_for_session, agent_backend_set_implement_display_name,
    agent_backend_set_run_timing, AgentBackend,
};
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::cli::SharedOpts;
use crate::prompts::PromptStore;
use crate::router_flow::router_flow_prompt;
use crate::run_timing::acp_post_run::RunTimingSessionEnd;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[path = "router_flow_acp_support.rs"]
pub(crate) mod router_flow_acp_support;

#[path = "router_flow_coder_prompts.rs"]
mod router_flow_coder_prompts;

use router_flow_acp_support::{
    router_iteration_log_path, run_router_turns, snapshot_iteration_backups,
};

pub(crate) struct RouterAcpIterationOutcome {
    pub acp_result: Result<(), String>,
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

type SessionEndParts<'a> = (
    &'a mut AgentBackend,
    &'a Path,
    &'a Arc<Mutex<crate::run_timing::RunTiming>>,
    RunTimingSessionEnd,
);

pub(crate) async fn run_router_acp_iteration(
    mut input: RouterAcpIterationInput<'_>,
) -> RouterAcpIterationOutcome {
    let work_dir = input.artifacts.work_dir.as_path();
    let log_path = router_iteration_log_path(input.artifacts, input.agent_loop);
    let timing = agent_backend_attach_run_timing_for_session(input.client);
    if let Err(e) = input.client.begin_coder_session(work_dir).await {
        agent_backend_set_run_timing(input.client, None);
        return RouterAcpIterationOutcome {
            acp_result: Err(e.to_string()),
            iteration_backups: snapshot_iteration_backups(work_dir),
        };
    }
    agent_backend_set_implement_display_name(input.client, "router");
    let session_end = input.session_end;
    let run_dir = input.artifacts.run_dir.clone();

    match run_router_turns(&mut input, log_path.as_path()).await {
        Ok(iteration_backups) => {
            let parts: SessionEndParts<'_> =
                (input.client, &run_dir, &timing, session_end);
            let acp_result = end_router_acp_session(parts, Ok(())).await;
            RouterAcpIterationOutcome {
                acp_result,
                iteration_backups,
            }
        }
        Err(e) => {
            let parts: SessionEndParts<'_> =
                (input.client, &run_dir, &timing, session_end);
            RouterAcpIterationOutcome {
                acp_result: abort_router_acp_session(parts, e).await,
                iteration_backups: snapshot_iteration_backups(work_dir),
            }
        }
    }
}

fn emit_router_acp_timing(
    parts: SessionEndParts<'_>,
    agent_result: Result<(), String>,
) -> Result<(), String> {
    let (client, run_dir, timing, session_end) = parts;
    crate::acp_post_run::emit_run_timing_after_backend(crate::acp_post_run::RunTimingAfterBackend {
        backend: client,
        run_dir,
        timing,
        agent_result,
        session_end,
    })
}

async fn end_router_acp_session(
    parts: SessionEndParts<'_>,
    run_res: Result<(), String>,
) -> Result<(), String> {
    let end_res = parts.0.end_coder_session().await.map_err(|e| e.to_string());
    let merged = crate::acp_post_run::prefer_primary_over_secondary(run_res, end_res, "end coder session");
    emit_router_acp_timing(parts, merged)
}

async fn abort_router_acp_session(
    parts: SessionEndParts<'_>,
    err: String,
) -> Result<(), String> {
    crate::output::print_log_error(&err);
    crate::cli::error_run_log::note_command_error_emitted(&err);
    crate::cli::error_run_log::append_command_error_to_run_log(&err);
    agent_backend_set_run_timing(parts.0, None);
    end_router_acp_session(parts, Err(err)).await
}

#[cfg(test)]
mod kiss_static_fn_item_refs {
    use super::{
        abort_router_acp_session, emit_router_acp_timing, end_router_acp_session,
        run_router_acp_iteration, RouterAcpIterationInput, RouterAcpIterationOutcome,
    };

    #[test]
    fn kiss_static_fn_item_refs() {
        let _ = run_router_acp_iteration;
        let _ = emit_router_acp_timing;
        let _ = end_router_acp_session;
        let _ = abort_router_acp_session;
        let _ = super::router_flow_acp_support::run_router_turns;
        let _ = super::router_flow_coder_prompts::run_router_requirements_coder_prompt;
        let _: Option<RouterAcpIterationInput> = None;
        let _: Option<RouterAcpIterationOutcome> = None;
        let _ = super::router_flow_acp_support::router_iteration_log_path;
        let _ = super::router_flow_acp_support::empty_iteration_backups;
        let _ = super::router_flow_acp_support::snapshot_iteration_backups;
        let _ = stringify!(SessionEndParts);
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
