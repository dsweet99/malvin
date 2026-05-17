use std::collections::HashMap;

use super::review_attempt_kernel::{
    REVIEW_WRITE_INNER_RETRY_CAP, REVIEW_WRITE_MISSING_ARTIFACT_MSG,
    REVIEW_WRITE_MISSING_ARTIFACT_RETRY_MSG,
};
use super::review_loop_helpers::run_concerns_and_check_abort_impl;
use super::review_write_retry::{
    ReviewTwoPromptSession, ReviewWriteInnerOutcome, run_reviewers_spawn_then_review_write,
};
use super::{Orchestrator, WorkflowError};

struct CodeReviewAttempt<'a> {
    context: &'a HashMap<String, String>,
    attempt: usize,
}

pub(super) async fn run_code_review_phase(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    let max_loops = orchestrator.config.max_loops.max(1);
    for attempt in 1..=max_loops {
        let ctx = CodeReviewAttempt { context, attempt };
        match code_review_single_attempt(orchestrator, ctx).await? {
            CodeReviewAttemptOutcome::Lgtm => return Ok(()),
            CodeReviewAttemptOutcome::NotLgtm => {}
            CodeReviewAttemptOutcome::MissingArtifactReview => {
                if attempt >= max_loops {
                    return Err(WorkflowError(format!(
                        "review: {REVIEW_WRITE_MISSING_ARTIFACT_MSG} after retries"
                    )));
                }
                (orchestrator.progress_callback)(REVIEW_WRITE_MISSING_ARTIFACT_RETRY_MSG);
            }
        }
    }
    Err(WorkflowError(
        "Did not receive LGTM for review within max loops.".to_string(),
    ))
}

enum CodeReviewAttemptOutcome {
    Lgtm,
    NotLgtm,
    MissingArtifactReview,
}

async fn code_review_single_attempt(
    orchestrator: &mut Orchestrator<'_>,
    ctx: CodeReviewAttempt<'_>,
) -> Result<CodeReviewAttemptOutcome, WorkflowError> {
    (orchestrator.progress_callback)(&format!("Review (attempt {})", ctx.attempt));

    let outcome = {
        let Orchestrator {
            client,
            prompts,
            artifacts,
            session_dotfile_backups,
            progress_callback,
            ..
        } = orchestrator;
        run_reviewers_spawn_then_review_write(
            ReviewTwoPromptSession {
                client,
                prompts,
                artifacts,
                session_dotfile_backups,
                context: ctx.context,
                attempt: ctx.attempt,
                skip_repo_style: false,
            },
            REVIEW_WRITE_INNER_RETRY_CAP,
            || {
                progress_callback(REVIEW_WRITE_MISSING_ARTIFACT_RETRY_MSG);
            },
        )
        .await?
    };
    match outcome {
        ReviewWriteInnerOutcome::Lgtm => return Ok(CodeReviewAttemptOutcome::Lgtm),
        ReviewWriteInnerOutcome::MissingArtifactExhausted => {
            return Ok(CodeReviewAttemptOutcome::MissingArtifactReview);
        }
        ReviewWriteInnerOutcome::NotLgtm => {}
    }

    let concern_suffix = format!("review_attempt_{}", ctx.attempt);
    run_concerns_and_check_abort_impl(orchestrator, ctx.attempt, &concern_suffix, ctx.context)
        .await?;
    Ok(CodeReviewAttemptOutcome::NotLgtm)
}

#[cfg(test)]
mod tests {
    use crate::review_sync::is_lgtm_str;

    #[test]
    fn sync_review_file_for_attempt_regression_still_promotes_workspace_lgtm() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifact = tmp.path().join("artifact_review.md");
        let workspace = tmp.path().join("workspace_review.md");
        std::fs::write(&workspace, "LGTM\n").expect("workspace lgtm");
        let synced =
            crate::review_sync::sync_review_file_for_attempt(&artifact, &workspace).expect("sync");
        assert!(
            synced.as_deref().is_some_and(is_lgtm_str),
            "legacy sync path must still promote workspace LGTM when artifact empty"
        );
    }

    #[test]
    fn kiss_stringify_review_loop_units() {
        let _ = stringify!(super::run_code_review_phase);
        let _ = stringify!(crate::review_sync::sync_review_file_for_attempt);
    }
}
