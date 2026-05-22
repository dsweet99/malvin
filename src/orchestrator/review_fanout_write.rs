use std::collections::HashMap;

use crate::acp::AgentClient;
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::prompts::PromptStore;

use super::WorkflowError;
use super::constants::REVIEW_WRITE_FILE;
use super::review_attempt_kernel::artifact_review_lgtm_after_review_write;
use super::review_fanout_run::{ReviewWriteCoderSession, run_review_write_coder_session};

#[derive(Debug)]
pub enum ReviewAttemptFinish {
    Lgtm,
    NotLgtm,
    MissingArtifact,
}

/// # Errors
///
/// Returns [`WorkflowError`] when `result.md` contains an `ABORT:` line.
pub fn fail_on_abort_for_artifacts(artifacts: &RunArtifacts) -> Result<(), WorkflowError> {
    if let Some(abort_msg) = super::check_abort(&artifacts.artifact_result_md()) {
        return Err(WorkflowError(format!("ABORT: {abort_msg}")));
    }
    Ok(())
}

pub struct FinishReviewWriteInput<'a> {
    pub client: &'a mut AgentClient,
    pub prompts: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
    pub context: &'a HashMap<String, String>,
    pub attempt: usize,
    pub log_attempt: usize,
    pub skip_repo_style: bool,
}

/// # Errors
///
/// Returns [`WorkflowError`] when the coder session, restore, or review read fails, or on `ABORT:`.
pub async fn finish_review_write_attempt(
    input: FinishReviewWriteInput<'_>,
) -> Result<ReviewAttemptFinish, WorkflowError> {
    let FinishReviewWriteInput {
        client,
        prompts,
        artifacts,
        session_dotfile_backups,
        context,
        attempt,
        log_attempt,
        skip_repo_style,
    } = input;
    let stdout_bracket_label = if skip_repo_style {
        None
    } else {
        Some(REVIEW_WRITE_FILE)
    };
    run_review_write_coder_session(ReviewWriteCoderSession {
        client,
        prompts,
        artifacts,
        session_dotfile_backups,
        context,
        attempt,
        log_attempt,
        skip_repo_style,
        stdout_bracket_label,
    })
    .await?;
    fail_on_abort_for_artifacts(artifacts)?;
    match artifact_review_lgtm_after_review_write(artifacts)? {
        None => Ok(ReviewAttemptFinish::MissingArtifact),
        Some(true) => Ok(ReviewAttemptFinish::Lgtm),
        Some(false) => Ok(ReviewAttemptFinish::NotLgtm),
    }
}

#[cfg(test)]
mod tests {
    use super::{ReviewAttemptFinish, fail_on_abort_for_artifacts, finish_review_write_attempt};
    use crate::artifacts::create_run_artifacts_from_text;
    use crate::orchestrator::orchestrator_test_support::{
        empty_dotfile_backups, no_session_client, workflow_ctx_for_smoke,
    };
    use crate::orchestrator::review_fanout_run::{
        ReviewWriteCoderSession, run_review_write_coder_session,
    };

    #[test]
    fn fail_on_abort_ok_when_result_has_no_abort() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = create_run_artifacts_from_text("rfw_smoke", Some(tmp.path())).expect("art");
        std::fs::write(artifacts.artifact_result_md(), "ok\n").expect("result");
        fail_on_abort_for_artifacts(&artifacts).expect("no abort");
    }

    #[test]
    fn fail_on_abort_err_when_result_contains_abort() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = create_run_artifacts_from_text("rfw_smoke", Some(tmp.path())).expect("art");
        std::fs::write(artifacts.artifact_result_md(), "ABORT: stop\n").expect("result");
        let err = fail_on_abort_for_artifacts(&artifacts).expect_err("abort");
        assert_eq!(err.0, "ABORT: stop");
    }

    #[test]
    fn review_attempt_finish_variants_are_distinct() {
        assert_ne!(
            std::mem::discriminant(&ReviewAttemptFinish::Lgtm),
            std::mem::discriminant(&ReviewAttemptFinish::NotLgtm)
        );
        assert_ne!(
            std::mem::discriminant(&ReviewAttemptFinish::NotLgtm),
            std::mem::discriminant(&ReviewAttemptFinish::MissingArtifact)
        );
    }

    #[tokio::test]
    async fn finish_review_write_errors_when_no_coder_session() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (artifacts, store, ctx) = workflow_ctx_for_smoke(&tmp, "rfw_smoke");
        let mut client = no_session_client();
        let backups = empty_dotfile_backups();
        let err = finish_review_write_attempt(super::FinishReviewWriteInput {
            client: &mut client,
            prompts: &store,
            artifacts: &artifacts,
            session_dotfile_backups: &backups,
            context: &ctx,
            attempt: 1,
            log_attempt: 1,
            skip_repo_style: true,
        })
        .await
        .expect_err("review_write without session");
        assert!(
            err.0.contains("begin_coder_session"),
            "unexpected: {}",
            err.0
        );
    }

    #[tokio::test]
    async fn run_review_write_coder_session_errors_when_no_coder_session() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let (artifacts, store, ctx) = workflow_ctx_for_smoke(&tmp, "rfw_smoke");
        let mut client = no_session_client();
        let backups = empty_dotfile_backups();
        let err = run_review_write_coder_session(ReviewWriteCoderSession {
            client: &mut client,
            prompts: &store,
            artifacts: &artifacts,
            session_dotfile_backups: &backups,
            context: &ctx,
            attempt: 1,
            log_attempt: 1,
            skip_repo_style: true,
            stdout_bracket_label: None,
        })
        .await
        .expect_err("expected no session");
        assert!(
            err.0.contains("begin_coder_session"),
            "unexpected: {}",
            err.0
        );
    }
}
