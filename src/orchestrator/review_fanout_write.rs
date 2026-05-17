use std::collections::HashMap;

use crate::acp::AgentClient;
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::prompts::PromptStore;

use super::WorkflowError;
use super::constants::REVIEW_WRITE_FILE;
use super::review_attempt_kernel::artifact_review_lgtm_after_review_write;
use super::review_fanout_run::{ReviewWriteCoderSession, run_review_write_coder_session};

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
    #[test]
    fn kiss_stringify_review_fanout_write_units() {
        let _ = stringify!(super::FinishReviewWriteInput);
        let _ = stringify!(super::finish_review_write_attempt);
        let _ = stringify!(super::fail_on_abort_for_artifacts);
        let _ = stringify!(super::ReviewAttemptFinish);
    }
}
