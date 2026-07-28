//! Explain Work coder turn (header + `explain_work.md`).

use crate::acp::{AgentError, CoderPromptOptions};
use crate::agent_backend::AgentBackend;
use crate::artifacts::SessionDotfileBackups;
use crate::cli::{SharedOpts, WorkflowCliOptions};
use crate::kpop_engine::KPopEnginePrepared;
use crate::prompts::{render_header, PromptError};
use crate::run_timing::TimingPhase;
use crate::prompt_stratification::{join_labeled_strata, PromptStratum};

pub(crate) struct ExplainWorkParams<'a> {
    #[allow(dead_code)] // kept for call-site symmetry with pre-reuse API
    pub shared: &'a SharedOpts,
    #[allow(dead_code)]
    pub workflow: WorkflowCliOptions,
    pub prepared: &'a KPopEnginePrepared,
    pub work_request: &'a str,
    #[allow(dead_code)]
    pub run_timing: &'a std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>,
    /// Open coder session reused across Review/Plan/Work (caller owns begin/end).
    pub client: &'a mut AgentBackend,
}

pub(crate) async fn run_explain_work(
    params: ExplainWorkParams<'_>,
) -> Result<SessionDotfileBackups, String> {
    let prepared = params.prepared;
    let work_dir = prepared.artifacts().work_dir.as_path();
    let session_dotfile_backups =
        SessionDotfileBackups::snapshot_after_ensuring_home_config(work_dir)?;
    let combined = build_work_prompt(prepared, params.work_request)?;
    let log_path = prepared.artifacts().log_path("explain_work");
    let prompt_result = params
        .client
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
    finalize_work_prompt(
        work_dir,
        &session_dotfile_backups,
        prompt_result,
    )
    .await
}

fn build_work_prompt(prepared: &KPopEnginePrepared, work_request: &str) -> Result<String, String> {
    let header = render_header(prepared.store(), prepared.context().as_map())
        .map_err(|e: PromptError| e.0)?;
    Ok(join_labeled_strata([
        (PromptStratum::WorkflowHeader, header.trim_end()),
        (PromptStratum::UserRequest, work_request.trim_end()),
    ]))
}

async fn finalize_work_prompt(
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
    session_dotfile_backups.restore_excluding_malvin_checks(work_dir)?;
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
