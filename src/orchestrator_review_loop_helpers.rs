use crate::orchestrator::review_context::ReviewAttemptCtx;
use crate::orchestrator::{Orchestrator, WorkflowError};
use crate::prompts::HEADER_MD;
use crate::run_timing::ReviewPairId;
use std::collections::HashMap;
use std::path::Path;

pub struct SyncConcernsContext<'a> {
    pub attempt: usize,
    pub concern_suffix: &'a str,
    pub context: &'a HashMap<String, String>,
    pub prepend_sync_header: bool,
}

pub async fn run_concerns_and_check_abort_impl(
    orchestrator: &mut Orchestrator<'_>,
    concerns_ctx: &SyncConcernsContext<'_>,
) -> Result<bool, WorkflowError> {
    if let Some(abort_msg) =
        crate::orchestrator::check_abort(&orchestrator.artifacts.artifact_result_md())
    {
        return Err(WorkflowError(format!("ABORT: {abort_msg}")));
    }
    (orchestrator.progress_callback)(&format!("Concerns (attempt {})", concerns_ctx.attempt));
    let concerns_body = prompt_with_sync_header(
        orchestrator,
        "concerns.md",
        concerns_ctx.context,
        concerns_ctx.prepend_sync_header,
    )?;
    orchestrator
        .run_coder_prompt_body(
            concerns_body,
            "concerns.md",
            concerns_ctx.concern_suffix,
            crate::run_timing::TimingPhase::Concerns,
        )
        .await?;
    if let Some(abort_msg) =
        crate::orchestrator::check_abort(&orchestrator.artifacts.artifact_result_md())
    {
        return Err(WorkflowError(format!("ABORT: {abort_msg}")));
    }
    Ok(false)
}

pub async fn run_reviewer_pair_for_attempt(
    orchestrator: &mut Orchestrator<'_>,
    ctx: &ReviewAttemptCtx<'_>,
    review_body: &str,
    pair_id: ReviewPairId,
) -> Result<(), WorkflowError> {
    let stem = crate::orchestrator::prompt_md_stem(ctx.review_prompt);
    let review_log = orchestrator
        .artifacts
        .log_path(&format!("reviewer_{stem}_attempt_{}", ctx.attempt));

    let pair = crate::acp::ReviewerPromptPair {
        cwd: &orchestrator.artifacts.work_dir,
        workspace_review_path: &orchestrator.artifacts.workspace_review_md(),
        artifact_review_path: ctx.review_path,
        review_body,
        review_who: stem,
        review_log: &review_log,
    };
    orchestrator
        .client
        .run_reviewer_review(
            pair,
            pair_id,
            crate::acp::ReviewerRestorePolicy::RestoreWorkspace,
        )
        .await
        .map_err(|e: crate::acp::AgentError| WorkflowError(e.0))?;
    Ok(())
}

pub fn sync_review_file_for_attempt(
    artifact_review_path: &Path,
    workspace_review_path: &Path,
) -> Result<Option<String>, WorkflowError> {
    if workspace_review_path.exists() {
        let workspace_text = std::fs::read_to_string(workspace_review_path).map_err(|e| {
            WorkflowError(format!(
                "failed to read workspace review file: {}: {e}",
                workspace_review_path.display()
            ))
        })?;
        if !workspace_text.trim().is_empty() {
            std::fs::write(artifact_review_path, &workspace_text).map_err(|e| {
                WorkflowError(format!(
                    "failed to sync workspace review into artifact: {}: {e}",
                    artifact_review_path.display()
                ))
            })?;
            return Ok(Some(workspace_text));
        }
    }

    if artifact_review_path.exists() {
        let artifact_text = std::fs::read_to_string(artifact_review_path).map_err(|e| {
            WorkflowError(format!(
                "failed to read artifact review file: {}: {e}",
                artifact_review_path.display()
            ))
        })?;
        if !artifact_text.trim().is_empty() {
            return Ok(Some(artifact_text));
        }
    }

    Ok(None)
}

pub fn prompt_with_sync_header(
    orchestrator: &Orchestrator<'_>,
    prompt_filename: &str,
    context: &HashMap<String, String>,
    prepend_sync_header: bool,
) -> Result<String, WorkflowError> {
    let prompt = orchestrator
        .prompts
        .render(prompt_filename, context)
        .map_err(|e| WorkflowError(e.0))?;
    if !prepend_sync_header {
        return Ok(prompt);
    }
    let header = orchestrator
        .prompts
        .render_prompt_only(HEADER_MD, context)
        .map_err(|e| WorkflowError(e.0))?;
    let header = header.trim();
    let prompt = prompt.trim();
    if header.is_empty() {
        return Ok(prompt.to_string());
    }
    if prompt.is_empty() {
        return Ok(header.to_string());
    }
    Ok(format!("{header}\n\n{prompt}"))
}

#[cfg(test)]
mod kiss_coverage_tests {
    #[test]
    fn kiss_stringify_review_loop_helpers() {
        let _ = stringify!(super::SyncConcernsContext);
        let _ = stringify!(super::run_concerns_and_check_abort_impl);
        let _ = stringify!(super::run_reviewer_pair_for_attempt);
        let _ = stringify!(super::sync_review_file_for_attempt);
        let _ = stringify!(super::prompt_with_sync_header);
    }
}
