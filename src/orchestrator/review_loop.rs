//! Review phase loop: reviewer, concerns.

use crate::acp::{AgentError, ReviewerPromptPair};
use crate::review_sync::is_lgtm_str;
use crate::run_timing::{ReviewPairId, TimingPhase};
use std::collections::HashMap;
use std::path::Path;

use super::Orchestrator;
use super::WorkflowError;
use super::check_abort;
use super::clear_review_file;
use super::prompt_md_stem;
use super::review_context::{ReviewAttemptCtx, ReviewPhaseArgs};

pub(super) async fn run_review_phase(
    orchestrator: &mut Orchestrator<'_>,
    phase: ReviewPhaseArgs<'_>,
) -> Result<(), WorkflowError> {
    let review_path = orchestrator.artifacts.artifact_review_md();

    for attempt in 1..=orchestrator.config.max_loops {
        let ctx = ReviewAttemptCtx {
            review_prompt: phase.review_prompt,
            progress_label: phase.progress_label,
            phase_id: phase.phase_id,
            attempt,
            review_path: &review_path,
            context: phase.context,
        };
        if review_phase_single_attempt(orchestrator, ctx).await? {
            return Ok(());
        }
    }
    Err(WorkflowError(format!(
        "Did not receive LGTM for {} within max loops.",
        phase.review_prompt
    )))
}

pub(super) async fn run_sync_check_loop(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    let review_path = orchestrator.artifacts.artifact_review_md();
    for attempt in 1..=orchestrator.config.max_loops {
        let ctx = ReviewAttemptCtx {
            review_prompt: "check_sync.md",
            progress_label: "CheckSync",
            phase_id: "check_sync",
            attempt,
            review_path: &review_path,
            context,
        };
        if run_sync_check_single_attempt(orchestrator, ctx).await? {
            return Ok(());
        }
    }
    Err(WorkflowError(
        "Did not receive LGTM for check_sync.md within max loops.".to_string(),
    ))
}

async fn review_phase_single_attempt(
    orchestrator: &mut Orchestrator<'_>,
    ctx: ReviewAttemptCtx<'_>,
) -> Result<bool, WorkflowError> {
    let workspace_review_path = orchestrator.artifacts.workspace_review_md();
    (orchestrator.progress_callback)(&format!("{} (attempt {})", ctx.progress_label, ctx.attempt));

    clear_review_file(ctx.review_path)
        .map_err(|e| WorkflowError(format!("failed to clear artifact review: {e}")))?;
    clear_review_file(&workspace_review_path)
        .map_err(|e| WorkflowError(format!("failed to clear workspace review: {e}")))?;

    let review_body = orchestrator
        .prompts
        .render(ctx.review_prompt, ctx.context)
        .map_err(|e| WorkflowError(e.0))?;

    let pair_id = match ctx.phase_id {
        "review_2" => ReviewPairId::Two,
        _ => ReviewPairId::One,
    };
    run_reviewer_pair_for_attempt(
        orchestrator,
        &ctx,
        &review_body,
        pair_id,
    )
    .await?;

    let lgtm_text = sync_review_file_for_attempt(ctx.review_path, &workspace_review_path)?;
    let lgtm = lgtm_text.as_deref().is_some_and(is_lgtm_str);
    if lgtm {
        orchestrator.fail_on_abort_result()?;
        return Ok(true);
    }
    run_concerns_and_check_abort(orchestrator, &ctx).await
}

async fn run_concerns_and_check_abort(
    orchestrator: &mut Orchestrator<'_>,
    ctx: &ReviewAttemptCtx<'_>,
) -> Result<bool, WorkflowError> {
    (orchestrator.progress_callback)(&format!("Concerns (attempt {})", ctx.attempt));
    orchestrator
        .run_coder_prompt(
            "concerns.md",
            ctx.context,
            &format!("{}_attempt_{}", ctx.phase_id, ctx.attempt),
            TimingPhase::Concerns,
        )
        .await?;
    if let Some(abort_msg) = check_abort(&orchestrator.artifacts.artifact_result_md()) {
        return Err(WorkflowError(format!("ABORT: {abort_msg}")));
    }
    Ok(false)
}

async fn run_sync_check_single_attempt(
    orchestrator: &mut Orchestrator<'_>,
    ctx: ReviewAttemptCtx<'_>,
) -> Result<bool, WorkflowError> {
    let workspace_review_path = orchestrator.artifacts.workspace_review_md();
    (orchestrator.progress_callback)(&format!("{} (attempt {})", ctx.progress_label, ctx.attempt));

    clear_review_file(ctx.review_path)
        .map_err(|e| WorkflowError(format!("failed to clear artifact review: {e}")))?;
    clear_review_file(&workspace_review_path)
        .map_err(|e| WorkflowError(format!("failed to clear workspace review: {e}")))?;

    orchestrator
        .run_coder_prompt(
            ctx.review_prompt,
            ctx.context,
            &format!("{}_attempt_{}", ctx.phase_id, ctx.attempt),
            TimingPhase::SyncCheck,
        )
        .await?;

    let lgtm_text = sync_review_file_for_attempt(ctx.review_path, &workspace_review_path)?;
    let lgtm = lgtm_text.as_deref().is_some_and(is_lgtm_str);
    if lgtm {
        orchestrator.fail_on_abort_result()?;
        return Ok(true);
    }
    run_sync_concerns_and_check_abort(orchestrator, &ctx).await
}

async fn run_sync_concerns_and_check_abort(
    orchestrator: &mut Orchestrator<'_>,
    ctx: &ReviewAttemptCtx<'_>,
) -> Result<bool, WorkflowError> {
    (orchestrator.progress_callback)(&format!("Concerns (attempt {})", ctx.attempt));
    orchestrator
        .run_coder_prompt(
            "concerns.md",
            ctx.context,
            &format!("{}_concerns_{}", ctx.phase_id, ctx.attempt),
            TimingPhase::Concerns,
        )
        .await?;
    if let Some(abort_msg) = check_abort(&orchestrator.artifacts.artifact_result_md()) {
        return Err(WorkflowError(format!("ABORT: {abort_msg}")));
    }
    Ok(false)
}

async fn run_reviewer_pair_for_attempt(
    orchestrator: &mut Orchestrator<'_>,
    ctx: &ReviewAttemptCtx<'_>,
    review_body: &str,
    pair_id: ReviewPairId,
) -> Result<(), WorkflowError> {
    let stem = prompt_md_stem(ctx.review_prompt);
    let review_log = orchestrator
        .artifacts
        .log_path(&format!("reviewer_{stem}_attempt_{}", ctx.attempt));

    let pair = ReviewerPromptPair {
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
        .map_err(|e: AgentError| WorkflowError(e.0))?;
    Ok(())
}

fn sync_review_file_for_attempt(
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
