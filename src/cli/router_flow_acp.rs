use crate::cli::{RouterOpts, SharedOpts};
use crate::router_flow::router_flow_prompt;
use malvin::agent_backend::{set_implement_display_name, SdkClient};
use malvin::artifacts::{RunArtifacts, SessionDotfileBackups};
use malvin::prompts::PromptStore;
use malvin::run_timing::acp_post_run::RunTimingSessionEnd;
use std::path::Path;
use std::sync::{Arc, Mutex};

#[path = "router_flow_acp_support.rs"]
pub(crate) mod router_flow_acp_support;

#[path = "router_flow_coder_prompts.rs"]
mod router_flow_coder_prompts;

pub(crate) use router_flow_acp_support::{router_iteration_log_path, RouterExitSummarize};

use router_flow_acp_support::{run_router_turns, snapshot_iteration_backups};
use router_flow_coder_prompts::run_router_summarize_coder_prompt;

pub(crate) enum RouterAcpIterationOutcome {
    Closed {
        acp_result: Result<(), String>,
        iteration_backups: SessionDotfileBackups,
    },
    Open {
        iteration_backups: SessionDotfileBackups,
        done: bool,
        timing: Arc<Mutex<malvin::run_timing::RunTiming>>,
    },
}

pub(crate) struct RouterAcpIterationInput<'a> {
    pub client: &'a mut SdkClient,
    pub artifacts: &'a RunArtifacts,
    pub prompt_store: &'a PromptStore,
    pub shared: &'a SharedOpts,
    pub router: &'a RouterOpts,
    pub agent_loop: usize,
    pub session_end: RunTimingSessionEnd,
    pub max_hypotheses: usize,
}

pub(crate) type SessionEndParts<'a> = (
    &'a mut SdkClient,
    &'a Path,
    &'a Arc<Mutex<malvin::run_timing::RunTiming>>,
    RunTimingSessionEnd,
);

pub(crate) async fn begin_coder_session_if_needed(
    client: &mut SdkClient,
    work_dir: &Path,
) -> Result<malvin::agent_backend::CoderSessionEnsure, String> {
    client
        .start_coder_session(work_dir)
        .await
        .map_err(|e| e.to_string())
}

pub(crate) async fn run_router_acp_open_iteration(
    mut input: RouterAcpIterationInput<'_>,
) -> RouterAcpIterationOutcome {
    let work_dir = input.artifacts.work_dir.as_path();
    let log_path = router_iteration_log_path(input.artifacts, input.agent_loop);
    let timing = input.client.attach_run_timing_for_session();
    set_implement_display_name(input.client, "router");
    let session_end = input.session_end;
    let run_dir = input.artifacts.run_dir.clone();
    match run_router_turns(&mut input, log_path.as_path()).await {
        Ok(turns) => RouterAcpIterationOutcome::Open {
            iteration_backups: turns.iteration_backups,
            done: turns.done,
            timing,
        },
        Err(e) => {
            let parts: SessionEndParts<'_> = (input.client, &run_dir, &timing, session_end);
            RouterAcpIterationOutcome::Closed {
                acp_result: abort_router_acp_session(parts, e).await,
                iteration_backups: snapshot_iteration_backups(work_dir),
            }
        }
    }
}

pub(crate) async fn finalize_router_acp_iteration(
    input: &mut RouterAcpIterationInput<'_>,
    timing: Arc<Mutex<malvin::run_timing::RunTiming>>,
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
            },
        )?;
        run_router_summarize_coder_prompt(input.client, &body, log_path.as_path()).await?;
    }
    let run_dir = input.artifacts.run_dir.clone();
    let parts: SessionEndParts<'_> = (input.client, run_dir.as_path(), &timing, input.session_end);
    match exit_summarize {
        RouterExitSummarize::Run => end_router_acp_session(parts, Ok(())).await,
        RouterExitSummarize::Skip => emit_router_acp_timing(parts, Ok(())),
    }
}

pub(crate) fn emit_router_acp_timing(
    parts: SessionEndParts<'_>,
    agent_result: Result<(), String>,
) -> Result<(), String> {
    let (client, run_dir, timing, session_end) = parts;
    malvin::acp_post_run::emit_run_timing_after_backend(
        malvin::acp_post_run::RunTimingAfterBackend {
            backend: client,
            run_dir,
            timing,
            agent_result,
            session_end,
        },
    )
}

pub(crate) async fn end_router_acp_session(
    parts: SessionEndParts<'_>,
    run_res: Result<(), String>,
) -> Result<(), String> {
    let end_res = parts.0.end_coder_session().await.map_err(|e| e.to_string());
    let merged =
        malvin::acp_post_run::prefer_primary_over_secondary(run_res, end_res, "end coder session");
    emit_router_acp_timing(parts, merged)
}

pub(crate) async fn abort_router_acp_session(
    parts: SessionEndParts<'_>,
    err: String,
) -> Result<(), String> {
    malvin::output::print_log_error(&err);
    crate::cli::error_run_log::note_command_error_emitted(&err);
    crate::cli::error_run_log::append_command_error_to_run_log(&err);
    parts.0.set_run_timing(None);
    end_router_acp_session(parts, Err(err)).await
}
