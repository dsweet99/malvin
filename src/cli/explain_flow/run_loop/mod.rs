mod review_lgtm;
mod phases;

pub(crate) use review_lgtm::explain_review_chat_is_lgtm;

use crate::acp::AgentError;
use crate::agent_backend::{agent_backend_set_run_timing, build_agent_backend, AgentBackend};
use crate::artifacts::SessionDotfileBackups;
use crate::cli::error_run_log;
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::run_timing::attach_new_run_timing;

use super::finish::emit_explain_startup;
use super::run_startup::{prepare_explain_kpop_run, ExplainKpopPrepared};
use super::{effective_explain_max_loops, ExplainArgs};
use phases::{run_explain_with_open_session, ExplainOpenSession};

pub async fn run_explain(
    explain: &mut ExplainArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<(), String> {
    let prepared = prepare_explain_run(explain, shared, workflow)?;
    error_run_log::set_command_error_run_dir(Some(prepared.inner.artifacts.run_dir.clone()));
    emit_explain_startup(shared, &prepared)?;
    let mut run_timing_slot = None;
    let run_timing = attach_new_run_timing(&mut run_timing_slot);
    let mut client = open_explain_backend(shared, workflow, &prepared, &run_timing)?;
    let work_dir = prepared.inner.artifacts.work_dir.as_path();
    let _session_backups = SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
    client
        .begin_coder_session(work_dir)
        .await
        .map_err(|e: AgentError| e.to_string())?;
    let max_outer = effective_explain_max_loops(explain.max_loops);
    let result = run_explain_with_open_session(ExplainOpenSession {
        explain,
        shared,
        workflow,
        prepared: &prepared,
        run_timing: &run_timing,
        client: &mut client,
        max_outer,
    })
    .await;
    let end = client.end_coder_session().await.map_err(|e: AgentError| e.to_string());
    match (result, end) {
        (Ok(ok), Ok(())) => Ok(ok),
        (Err(e), _) => Err(e),
        (Ok(()), Err(e)) => Err(e),
    }
}

fn open_explain_backend(
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
    prepared: &ExplainKpopPrepared,
    run_timing: &std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
) -> Result<AgentBackend, String> {
    let mut client = build_agent_backend(
        shared,
        workflow,
        shared.acp_stdout_markdown_enabled(),
        "explain",
    )
    .map_err(|e| e.to_string())?;
    agent_backend_set_run_timing(&mut client, Some(std::sync::Arc::clone(run_timing)));
    client.set_prompts_log_run_dir(Some(prepared.inner.artifacts.run_dir.clone()));
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    Ok(client)
}

fn prepare_explain_run(
    explain: &mut ExplainArgs,
    shared: &SharedOpts,
    workflow: WorkflowCliOptions,
) -> Result<ExplainKpopPrepared, String> {
    let _request = explain
        .request
        .as_ref()
        .ok_or_else(|| "malvin explain: missing required REQUEST (text or path)".to_string())?;
    let prepared = prepare_explain_kpop_run(
        explain.request.as_ref(),
        &explain.out_path,
        explain.out_path_explicit,
        super::run_startup::ExplainKpopPrepareOpts {
            workflow,
            model: &shared.model,
            git: shared.git,
        },
    )?;
    if explain.out_path_explicit {
        explain.out_path =
            crate::cli::default_output_path::path_relative_to_cwd(&prepared.tex_path)?;
    }
    Ok(prepared)
}

#[cfg(test)]
#[path = "../../explain_flow_run_loop_tests.rs"]
mod explain_flow_run_loop_tests;

#[cfg(test)]
mod run_loop_cov;
