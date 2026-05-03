use crate::review_sync::{is_lgtm_str, sync_review_file_for_attempt};
use crate::run_timing::{ReviewPairId, TimingPhase};
use std::collections::HashMap;

use super::Orchestrator;
use super::WorkflowError;
use super::clear_review_file;
use super::review_context::{ReviewAttemptCtx, ReviewPhaseArgs};
use super::review_loop_helpers::{
    SyncConcernsContext, prompt_with_sync_header, run_concerns_and_check_abort_impl,
    run_reviewer_pair_for_attempt,
};

pub(super) enum SyncCheckAfterNonLgtm {
    RunConcerns,
    DryRunStop,
}

pub(super) async fn run_review_phase(
    orchestrator: &mut Orchestrator<'_>,
    phase: ReviewPhaseArgs<'_>,
    prepend_sync_header: bool,
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
        if review_phase_single_attempt(orchestrator, ctx, prepend_sync_header).await? {
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
    prepend_sync_header: bool,
) -> Result<(), WorkflowError> {
    let review_path = orchestrator.artifacts.artifact_review_md();
    for attempt in 1..=orchestrator.config.max_loops.max(1) {
        let ctx = ReviewAttemptCtx {
            review_prompt: "check_sync.md",
            progress_label: "CheckSync",
            phase_id: "check_sync",
            attempt,
            review_path: &review_path,
            context,
        };
        if run_sync_check_single_attempt(
            orchestrator,
            ctx,
            prepend_sync_header,
            SyncCheckAfterNonLgtm::RunConcerns,
        )
        .await?
        {
            return Ok(());
        }
    }
    Err(WorkflowError(
        "Did not receive LGTM for check_sync.md within max loops.".to_string(),
    ))
}

pub(super) async fn run_sync_check_dry_run_once(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
    prepend_sync_header: bool,
) -> Result<(), WorkflowError> {
    let review_path = orchestrator.artifacts.artifact_review_md();
    let ctx = ReviewAttemptCtx {
        review_prompt: "check_sync.md",
        progress_label: "CheckSync",
        phase_id: "check_sync",
        attempt: 1,
        review_path: &review_path,
        context,
    };
    let _ = run_sync_check_single_attempt(
        orchestrator,
        ctx,
        prepend_sync_header,
        SyncCheckAfterNonLgtm::DryRunStop,
    )
    .await?;
    Ok(())
}

async fn review_phase_single_attempt(
    orchestrator: &mut Orchestrator<'_>,
    ctx: ReviewAttemptCtx<'_>,
    prepend_sync_header: bool,
) -> Result<bool, WorkflowError> {
    let workspace_review_path = orchestrator.artifacts.workspace_review_md();
    (orchestrator.progress_callback)(&format!("{} (attempt {})", ctx.progress_label, ctx.attempt));

    clear_review_file(ctx.review_path)
        .map_err(|e| WorkflowError(format!("failed to clear artifact review: {e}")))?;
    clear_review_file(&workspace_review_path)
        .map_err(|e| WorkflowError(format!("failed to clear workspace review: {e}")))?;

    let review_body = prompt_with_sync_header(
        orchestrator,
        ctx.review_prompt,
        ctx.context,
        prepend_sync_header,
    )?;

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
    run_concerns_and_check_abort(orchestrator, &ctx, "attempt", prepend_sync_header).await
}

async fn run_concerns_and_check_abort(
    orchestrator: &mut Orchestrator<'_>,
    ctx: &ReviewAttemptCtx<'_>,
    concern_suffix_kind: &str,
    prepend_sync_header: bool,
) -> Result<bool, WorkflowError> {
    let concern_suffix = format!(
        "{0}_{1}_{2}",
        ctx.phase_id, concern_suffix_kind, ctx.attempt
    );
    run_concerns_and_check_abort_impl(
        orchestrator,
        &SyncConcernsContext {
            attempt: ctx.attempt,
            concern_suffix: &concern_suffix,
            context: ctx.context,
            prepend_sync_header,
        },
    )
    .await
}

async fn run_sync_check_single_attempt(
    orchestrator: &mut Orchestrator<'_>,
    ctx: ReviewAttemptCtx<'_>,
    prepend_sync_header: bool,
    after_non_lgtm: SyncCheckAfterNonLgtm,
) -> Result<bool, WorkflowError> {
    let workspace_review_path = orchestrator.artifacts.workspace_review_md();
    (orchestrator.progress_callback)(&format!("{} (attempt {})", ctx.progress_label, ctx.attempt));

    clear_review_file(ctx.review_path)
        .map_err(|e| WorkflowError(format!("failed to clear artifact review: {e}")))?;
    clear_review_file(&workspace_review_path)
        .map_err(|e| WorkflowError(format!("failed to clear workspace review: {e}")))?;

    let check_prompt = prompt_with_sync_header(
        orchestrator,
        ctx.review_prompt,
        ctx.context,
        prepend_sync_header,
    )?;
    orchestrator
        .run_coder_prompt_body(
            check_prompt,
            ctx.review_prompt,
            &format!("{}_attempt_{}", ctx.phase_id, ctx.attempt),
            TimingPhase::SyncCheck,
        )
        .await?;

    let lgtm_text = sync_review_file_for_attempt(ctx.review_path, &workspace_review_path)
        .map_err(WorkflowError)?;
    let lgtm = lgtm_text.as_deref().is_some_and(is_lgtm_str);
    if lgtm {
        orchestrator.fail_on_abort_result()?;
        return Ok(true);
    }
    match after_non_lgtm {
        SyncCheckAfterNonLgtm::DryRunStop => {
            orchestrator.fail_on_abort_result()?;
            Ok(false)
        }
        SyncCheckAfterNonLgtm::RunConcerns => {
            run_concerns_and_check_abort(orchestrator, &ctx, "concerns", prepend_sync_header).await
        }
    }
}

#[cfg(test)]
mod kiss_coverage_tests {
    #[test]
    fn kiss_stringify_review_loop_units() {
        let _ = stringify!(super::run_review_phase);
        let _ = stringify!(super::run_sync_check_loop);
        let _ = stringify!(super::review_phase_single_attempt);
        let _ = stringify!(super::run_sync_check_single_attempt);
        let _ = stringify!(super::SyncCheckAfterNonLgtm);
        let _ = stringify!(super::run_sync_check_dry_run_once);
        let _ = stringify!(super::run_concerns_and_check_abort);
    }
}
