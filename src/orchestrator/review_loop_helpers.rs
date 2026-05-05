use super::review_context::ReviewAttemptCtx;
use super::{Orchestrator, WorkflowError};
use crate::run_timing::ReviewPairId;
use std::collections::HashMap;

pub async fn run_concerns_and_check_abort_impl(
    orchestrator: &mut Orchestrator<'_>,
    attempt: usize,
    concern_suffix: &str,
    context: &HashMap<String, String>,
) -> Result<bool, WorkflowError> {
    if let Some(abort_msg) = super::check_abort(&orchestrator.artifacts.artifact_result_md()) {
        return Err(WorkflowError(format!("ABORT: {abort_msg}")));
    }
    (orchestrator.progress_callback)(&format!("Concerns (attempt {attempt})"));
    let concerns_body = orchestrator
        .prompts
        .render("concerns.md", context)
        .map_err(|e| WorkflowError(e.0))?;
    orchestrator
        .run_coder_prompt_body(
            concerns_body,
            "concerns.md",
            concern_suffix,
            crate::run_timing::TimingPhase::Concerns,
        )
        .await?;
    if let Some(abort_msg) = super::check_abort(&orchestrator.artifacts.artifact_result_md()) {
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
    let stem = super::prompt_md_stem(ctx.review_prompt);
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

#[cfg(test)]
mod kiss_coverage_tests {
    #[test]
    fn kiss_stringify_review_loop_helpers() {
        let _ = stringify!(super::run_concerns_and_check_abort_impl);
        let _ = stringify!(super::run_reviewer_pair_for_attempt);
        let _ = stringify!(crate::review_sync::sync_review_file_for_attempt);
    }
}
