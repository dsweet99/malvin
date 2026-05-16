use std::collections::HashMap;
use std::path::Path;

use crate::review_sync::{is_lgtm_str, sync_review_file_for_attempt};

use super::review_fanout_desc::{
    load_review_description_lines, reviewers_attempt_dir, verify_reviewer_output_files,
};
use super::review_fanout_run::{FanoutPrepareInput, run_review_fanout_jobs};
use super::review_fanout_write::run_review_write_prompt;
use super::review_loop_helpers::run_concerns_and_check_abort_impl;
use super::{Orchestrator, WorkflowError, clear_review_file};

struct CodeReviewAttempt<'a> {
    context: &'a HashMap<String, String>,
    descriptions: &'a [String],
    attempt: usize,
    review_path: &'a Path,
}

pub(super) async fn run_code_review_phase(
    orchestrator: &mut Orchestrator<'_>,
    context: &HashMap<String, String>,
) -> Result<(), WorkflowError> {
    let review_path = orchestrator.artifacts.artifact_review_md();
    let descriptions = load_review_description_lines(orchestrator.prompts)?;
    if descriptions.is_empty() {
        return Err(WorkflowError(
            "review_descriptions.md has no non-empty lines".to_string(),
        ));
    }

    for attempt in 1..=orchestrator.config.max_loops.max(1) {
        let ctx = CodeReviewAttempt {
            context,
            descriptions: &descriptions,
            attempt,
            review_path: &review_path,
        };
        if code_review_single_attempt(orchestrator, ctx).await? {
            return Ok(());
        }
    }
    Err(WorkflowError(
        "Did not receive LGTM for review within max loops.".to_string(),
    ))
}

async fn code_review_single_attempt(
    orchestrator: &mut Orchestrator<'_>,
    ctx: CodeReviewAttempt<'_>,
) -> Result<bool, WorkflowError> {
    let workspace_review_path = orchestrator.artifacts.workspace_review_md();
    (orchestrator.progress_callback)(&format!("Review (attempt {})", ctx.attempt));

    clear_review_file(ctx.review_path)
        .map_err(|e| WorkflowError(format!("failed to clear artifact review: {e}")))?;
    clear_review_file(&workspace_review_path)
        .map_err(|e| WorkflowError(format!("failed to clear workspace review: {e}")))?;

    let reviewers_subdir = reviewers_attempt_dir(&orchestrator.artifacts.run_dir, ctx.attempt);
    let fanout = FanoutPrepareInput {
        store: orchestrator.prompts,
        artifacts: orchestrator.artifacts,
        context: ctx.context,
        descriptions: ctx.descriptions,
        reviewers_subdir: &reviewers_subdir,
        attempt: ctx.attempt,
    };
    run_review_fanout_jobs(&*orchestrator.client, fanout).await?;
    verify_reviewer_output_files(&reviewers_subdir, ctx.descriptions.len())?;
    run_review_write_prompt(orchestrator, ctx.context, &reviewers_subdir, ctx.attempt).await?;

    let review_text = sync_review_file_for_attempt(ctx.review_path, &workspace_review_path)
        .map_err(WorkflowError)?;
    if review_text.as_deref().is_some_and(is_lgtm_str) {
        orchestrator.fail_on_abort_result()?;
        return Ok(true);
    }

    let concern_suffix = format!("review_attempt_{}", ctx.attempt);
    run_concerns_and_check_abort_impl(
        orchestrator,
        ctx.attempt,
        &concern_suffix,
        ctx.context,
    )
    .await
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
        let synced =
            super::sync_review_file_for_attempt(&artifact, &workspace).expect("sync after write");
        assert_eq!(synced, None, "missing review files must not be a read error");
        assert!(
            !synced.as_deref().is_some_and(is_lgtm_str),
            "absent review must not count as LGTM"
        );
    }

    #[test]
    fn regression_workspace_lgtm_syncs_into_artifact_before_lgtm_gate() {
        let dir = tempfile::tempdir().expect("tempdir");
        let artifact = dir.path().join("run").join("review.md");
        let workspace = dir.path().join("workspace_review.md");
        std::fs::create_dir_all(artifact.parent().expect("parent")).expect("mkdir");
        std::fs::write(&workspace, "LGTM\n").expect("workspace lgtm");
        let synced =
            super::sync_review_file_for_attempt(&artifact, &workspace).expect("sync workspace");
        assert!(
            synced.as_deref().is_some_and(is_lgtm_str),
            "workspace LGTM must be visible through sync: {synced:?}"
        );
        assert_eq!(std::fs::read_to_string(&artifact).expect("artifact"), "LGTM\n");
    }

    #[test]
    fn kiss_stringify_review_loop_units() {
        let _ = stringify!(super::run_code_review_phase);
        let _ = stringify!(super::code_review_single_attempt);
        let _ = stringify!(crate::review_sync::sync_review_file_for_attempt);
    }
}
