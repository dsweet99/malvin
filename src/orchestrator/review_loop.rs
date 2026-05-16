use std::collections::HashMap;

use super::review_attempt_kernel::{
    ReviewAttemptKernelInput, REVIEW_WRITE_MISSING_ARTIFACT_MSG,
    ensure_artifact_review_after_review_write, is_missing_artifact_review_error,
    load_review_descriptions_for_kernel, review_attempt_is_lgtm, run_review_fanout_prefix,
};
use super::review_fanout_write::{ReviewWriteCoderSession, run_review_write_coder_session};
use super::review_loop_helpers::run_concerns_and_check_abort_impl;
use super::{Orchestrator, WorkflowError};

struct CodeReviewAttempt<'a> {
    context: &'a HashMap<String, String>,
    descriptions: &'a [String],
    attempt: usize,
}

pub(super) async fn run_code_review_phase(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    let descriptions = load_review_descriptions_for_kernel(orchestrator.prompts)?;

    let max_loops = orchestrator.config.max_loops.max(1);
    for attempt in 1..=max_loops {
        let ctx = CodeReviewAttempt {
            context,
            descriptions: &descriptions,
            attempt,
        };
        match code_review_single_attempt(orchestrator, ctx).await? {
            CodeReviewAttemptOutcome::Lgtm => return Ok(()),
            CodeReviewAttemptOutcome::NotLgtm => {}
            CodeReviewAttemptOutcome::MissingArtifactReview => {
                if attempt >= max_loops {
                    return Err(WorkflowError(format!(
                        "review: {REVIEW_WRITE_MISSING_ARTIFACT_MSG} after retries"
                    )));
                }
                (orchestrator.progress_callback)(
                    "Review: review_write did not write artifact review, retrying",
                );
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

    let kernel = ReviewAttemptKernelInput {
        store: orchestrator.prompts,
        artifacts: orchestrator.artifacts,
        context: ctx.context,
        descriptions: ctx.descriptions,
        attempt: ctx.attempt,
    };
    let reviewers_subdir = run_review_fanout_prefix(&*orchestrator.client, &kernel).await?;
    run_review_write_coder_session(ReviewWriteCoderSession {
        client: orchestrator.client,
        prompts: orchestrator.prompts,
        artifacts: orchestrator.artifacts,
        session_dotfile_backups: &orchestrator.session_dotfile_backups,
        context: ctx.context,
        reviewers_subdir: &reviewers_subdir,
        attempt: ctx.attempt,
    })
    .await?;
    if let Err(err) = ensure_artifact_review_after_review_write(orchestrator.artifacts) {
        if is_missing_artifact_review_error(&err) {
            orchestrator.fail_on_abort_result()?;
            return Ok(CodeReviewAttemptOutcome::MissingArtifactReview);
        }
        return Err(err);
    }
    let lgtm = review_attempt_is_lgtm(orchestrator.artifacts)?;

    if lgtm {
        orchestrator.fail_on_abort_result()?;
        return Ok(CodeReviewAttemptOutcome::Lgtm);
    }

    let concern_suffix = format!("review_attempt_{}", ctx.attempt);
    run_concerns_and_check_abort_impl(
        orchestrator,
        ctx.attempt,
        &concern_suffix,
        ctx.context,
    )
    .await?;
    Ok(CodeReviewAttemptOutcome::NotLgtm)
}

#[cfg(test)]
mod tests {
    use crate::review_sync::is_lgtm_str;

    use super::super::review_fanout_desc::{
        embedded_review_description_job_count, reviewer_output_filename,
        verify_reviewer_output_files,
    };

    #[test]
    fn preflight_fails_when_fanout_mock_skips_reviewer_writes() {
        let job_count = embedded_review_description_job_count();
        let dir = tempfile::tempdir().expect("tempdir");
        let err = verify_reviewer_output_files(dir.path(), job_count).expect_err("no reviewer files");
        assert!(
            err.0.contains("missing reviewer output"),
            "expected missing-output error, got: {}",
            err.0
        );
        let path = dir.path().join(reviewer_output_filename(1));
        std::fs::write(&path, "Executive summary: ok\n\ntl;dr: ok\n").expect("write one file");
        let err = verify_reviewer_output_files(dir.path(), job_count).expect_err("partial outputs");
        assert!(
            err.0.contains("missing reviewer output"),
            "expected missing-output error for partial set, got: {}",
            err.0
        );
    }

    #[test]
    fn regression_missing_artifact_review_is_non_lgtm_not_read_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let artifact = dir.path().join("review.md");
        let workspace = dir.path().join("workspace_review.md");
        super::super::clear_review_file(&artifact).expect("clear");
        assert!(
            !artifact.exists(),
            "test setup: artifact review must be absent"
        );
        let synced = crate::review_sync::sync_review_file_for_attempt(&artifact, &workspace)
            .expect("sync after write");
        assert_eq!(synced, None, "missing review files must not be a read error");
        assert!(
            !synced.as_deref().is_some_and(is_lgtm_str),
            "absent review must not count as LGTM"
        );
    }

    #[test]
    fn kiss_stringify_review_loop_units() {
        let _ = stringify!(super::run_code_review_phase);
        let _ = stringify!(super::code_review_single_attempt);
        let _ = stringify!(crate::review_sync::sync_review_file_for_attempt);
    }
}
