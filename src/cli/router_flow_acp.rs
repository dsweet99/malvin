use crate::agent_backend::{
    AgentBackend, agent_backend_attach_run_timing_for_session, agent_backend_ensure_coder_session,
    agent_backend_set_implement_display_name, agent_backend_set_run_timing,
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

pub(crate) use router_flow_acp_support::RouterExitSummarize;

use router_flow_acp_support::{
    router_iteration_log_path, run_router_turns, snapshot_iteration_backups,
};
use router_flow_coder_prompts::run_router_summarize_coder_prompt;

pub(crate) struct RouterAcpIterationOutcome {
    pub acp_result: Result<(), String>,
    pub iteration_backups: SessionDotfileBackups,
    pub done: bool,
    pub session_alive: bool,
    pub timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
}

pub(crate) struct RouterAcpIterationInput<'a> {
    pub client: &'a mut AgentBackend,
    pub artifacts: &'a RunArtifacts,
    pub prompt_store: &'a PromptStore,
    pub shared: &'a SharedOpts,
    pub agent_loop: usize,
    pub session_end: RunTimingSessionEnd,
    pub max_hypotheses: usize,
}

pub(crate) type SessionEndParts<'a> = (
    &'a mut AgentBackend,
    &'a Path,
    &'a Arc<Mutex<crate::run_timing::RunTiming>>,
    RunTimingSessionEnd,
);

pub(crate) async fn begin_coder_session_if_needed(
    client: &mut AgentBackend,
    work_dir: &Path,
) -> Result<(), String> {
    agent_backend_ensure_coder_session(client, work_dir)
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn run_router_acp_open_iteration(
    mut input: RouterAcpIterationInput<'_>,
) -> RouterAcpIterationOutcome {
    let work_dir = input.artifacts.work_dir.as_path();
    let log_path = router_iteration_log_path(input.artifacts, input.agent_loop);
    let timing = agent_backend_attach_run_timing_for_session(input.client);
    if let Err(e) = begin_coder_session_if_needed(input.client, work_dir).await {
        agent_backend_set_run_timing(input.client, None);
        return RouterAcpIterationOutcome {
            acp_result: Err(e),
            iteration_backups: snapshot_iteration_backups(work_dir),
            done: false,
            session_alive: false,
            timing: None,
        };
    }
    agent_backend_set_implement_display_name(input.client, "router");
    let session_end = input.session_end;
    let run_dir = input.artifacts.run_dir.clone();
    match run_router_turns(&mut input, log_path.as_path()).await {
        Ok(turns) => RouterAcpIterationOutcome {
            acp_result: Ok(()),
            iteration_backups: turns.iteration_backups,
            done: turns.done,
            session_alive: true,
            timing: Some(timing),
        },
        Err(e) => {
            let parts: SessionEndParts<'_> = (input.client, &run_dir, &timing, session_end);
            RouterAcpIterationOutcome {
                acp_result: abort_router_acp_session(parts, e).await,
                iteration_backups: snapshot_iteration_backups(work_dir),
                done: false,
                session_alive: false,
                timing: None,
            }
        }
    }
}

pub(crate) async fn finalize_router_acp_iteration(
    input: &mut RouterAcpIterationInput<'_>,
    timing: Arc<Mutex<crate::run_timing::RunTiming>>,
    exit_summarize: RouterExitSummarize,
) -> Result<(), String> {
    let log_path = router_iteration_log_path(input.artifacts, input.agent_loop);
    if matches!(exit_summarize, RouterExitSummarize::Run) {
        let model = input.shared.model.canonical();
        let body = router_flow_prompt::build_router_summarize_prompt(
            router_flow_prompt::RouterSummarizePromptInput {
                store: input.prompt_store,
                artifacts: input.artifacts,
                model: &model,
                git: input.shared.git,
            },
        )?;
        run_router_summarize_coder_prompt(input.client, &body, log_path.as_path()).await?;
    }
    let keep_session = true;
    let run_dir = input.artifacts.run_dir.clone();
    let parts: SessionEndParts<'_> = (input.client, run_dir.as_path(), &timing, input.session_end);
    match (exit_summarize, keep_session) {
        (RouterExitSummarize::Run, _) | (RouterExitSummarize::Skip, false) => {
            end_router_acp_session(parts, Ok(())).await
        }
        (RouterExitSummarize::Skip, true) => emit_router_acp_timing(parts, Ok(())),
    }
}

pub(crate) fn emit_router_acp_timing(
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

pub(crate) async fn end_router_acp_session(
    parts: SessionEndParts<'_>,
    run_res: Result<(), String>,
) -> Result<(), String> {
    let end_res = parts.0.end_coder_session().await.map_err(|e| e.to_string());
    let merged =
        crate::acp_post_run::prefer_primary_over_secondary(run_res, end_res, "end coder session");
    emit_router_acp_timing(parts, merged)
}

pub(crate) async fn abort_router_acp_session(
    parts: SessionEndParts<'_>,
    err: String,
) -> Result<(), String> {
    crate::output::print_log_error(&err);
    crate::cli::error_run_log::note_command_error_emitted(&err);
    crate::cli::error_run_log::append_command_error_to_run_log(&err);
    agent_backend_set_run_timing(parts.0, None);
    end_router_acp_session(parts, Err(err)).await
}
