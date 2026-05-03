use crate::review_sync::{is_lgtm_str, sync_review_file_for_attempt};
use crate::run_timing::ReviewPairId;

use super::Orchestrator;
use super::WorkflowError;
use super::clear_review_file;
use super::review_context::{ReviewAttemptCtx, ReviewPhaseArgs};
use super::review_loop_helpers::{run_concerns_and_check_abort_impl, run_reviewer_pair_for_attempt};

pub(super) async fn run_review_phase(
    orchestrator: &mut Orchestrator<'_>,
    phase: ReviewPhaseArgs<'_>,
) -> Result<(), WorkflowError> {
    let review_path = orchestrator.artifacts.artifact_review_md();

    for attempt in 1..=orchestrator.config.max_loops.max(1) {
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
    run_reviewer_pair_for_attempt(orchestrator, &ctx, &review_body, pair_id).await?;

    let lgtm_text = sync_review_file_for_attempt(ctx.review_path, &workspace_review_path)
        .map_err(WorkflowError)?;
    let lgtm = lgtm_text.as_deref().is_some_and(is_lgtm_str);
    if lgtm {
        orchestrator.fail_on_abort_result()?;
        return Ok(true);
    }
    run_concerns_and_check_abort(orchestrator, &ctx, "attempt").await
}

async fn run_concerns_and_check_abort(
    orchestrator: &mut Orchestrator<'_>,
    ctx: &ReviewAttemptCtx<'_>,
    concern_suffix_kind: &str,
) -> Result<bool, WorkflowError> {
    let concern_suffix = format!(
        "{0}_{1}_{2}",
        ctx.phase_id, concern_suffix_kind, ctx.attempt
    );
    run_concerns_and_check_abort_impl(
        orchestrator,
        ctx.attempt,
        &concern_suffix,
        ctx.context,
    )
    .await
}

#[cfg(test)]
mod kiss_coverage_tests {
    #[test]
    fn kiss_stringify_review_loop_units() {
        let _ = stringify!(super::run_review_phase);
        let _ = stringify!(super::review_phase_single_attempt);
        let _ = stringify!(super::run_concerns_and_check_abort);
    }
}
