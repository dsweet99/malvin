//! Explain Work coder turn (header + `explain_work.md`).

use crate::acp::{AgentError, CoderPromptOptions};
use crate::agent_backend::{agent_backend_set_run_timing, build_agent_backend, AgentBackend};
use crate::artifacts::SessionDotfileBackups;
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::kpop_engine::KPopEnginePrepared;
use crate::prompts::{render_header, PromptError};
use crate::run_timing::TimingPhase;
use crate::prompt_stratification::{join_labeled_strata, PromptStratum};

pub(crate) struct ExplainWorkParams<'a> {
    pub shared: &'a SharedOpts,
    pub workflow: WorkflowCliOptions,
    pub prepared: &'a KPopEnginePrepared,
    pub work_request: &'a str,
    pub run_timing: &'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
}

pub(crate) async fn run_explain_work(
    params: ExplainWorkParams<'_>,
) -> Result<SessionDotfileBackups, String> {
    let prepared = params.prepared;
    let work_dir = prepared.artifacts().work_dir.as_path();
    let mut client = open_work_backend(&params)?;
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
    let combined = build_work_prompt(prepared, params.work_request)?;
    let log_path = prepared.artifacts().log_path("explain_work");
    client
        .begin_coder_session(work_dir)
        .await
        .map_err(|e| e.to_string())?;
    let prompt_result = client
        .run_coder_prompt(
            &combined,
            log_path.as_path(),
            "explain_work",
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::Implement),
                ..Default::default()
            },
        )
        .await;
    finalize_work_session(
        &mut client,
        work_dir,
        &session_dotfile_backups,
        prompt_result,
    )
    .await
}

fn open_work_backend(params: &ExplainWorkParams<'_>) -> Result<AgentBackend, String> {
    let mut client = build_agent_backend(
        params.shared,
        params.workflow,
        params.shared.acp_stdout_markdown_enabled(),
        "explain",
    )
    .map_err(|e| e.to_string())?;
    agent_backend_set_run_timing(&mut client, Some(std::sync::Arc::clone(params.run_timing)));
    client.set_prompts_log_run_dir(Some(params.prepared.artifacts().run_dir.clone()));
    client.ensure_authenticated().map_err(|e| e.to_string())?;
    Ok(client)
}

fn build_work_prompt(prepared: &KPopEnginePrepared, work_request: &str) -> Result<String, String> {
    let header = render_header(prepared.store(), prepared.context().as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(join_labeled_strata([
        (PromptStratum::WorkflowHeader, header.trim_end()),
        (PromptStratum::UserRequest, work_request.trim_end()),
    ]))
}

async fn finalize_work_session(
    client: &mut AgentBackend,
    work_dir: &std::path::Path,
    session_dotfile_backups: &SessionDotfileBackups,
    prompt_result: Result<(), AgentError>,
) -> Result<SessionDotfileBackups, String> {
    let post_agent_backups = if prompt_result.is_ok() {
        Some(SessionDotfileBackups::snapshot_after_ensuring_home_config(
            work_dir,
        )?)
    } else {
        None
    };
    if let Err(restore_err) = session_dotfile_backups.restore_excluding_malvin_checks(work_dir) {
        client.end_coder_session().await.ok();
        return Err(restore_err);
    }
    client
        .end_coder_session()
        .await
        .map_err(|e| e.to_string())?;
    prompt_result.map_err(|e| e.to_string())?;
    let progress = post_agent_backups.unwrap_or_else(|| session_dotfile_backups.clone());
    Ok(crate::artifacts::merge_and_sanitize_for_gate_restore(
        session_dotfile_backups,
        &progress,
        work_dir,
    ))
}

#[cfg(test)]
mod work_cov;
