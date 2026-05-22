use std::collections::HashMap;

use crate::acp::AgentClient;
use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::prompts::PromptStore;

use super::WorkflowError;
use super::review_attempt_kernel::{
    clear_review_attempt_artifacts, ensure_review_prep_after_reviewers_spawn,
};
use super::review_fanout_run::{ReviewersSpawnCoderSession, run_reviewers_spawn_coder_session};
use super::review_fanout_write::{
    FinishReviewWriteInput, ReviewAttemptFinish, finish_review_write_attempt,
};

#[derive(Debug)]
pub enum ReviewWriteInnerOutcome {
    Lgtm,
    NotLgtm,
    MissingArtifactExhausted,
}

pub struct ReviewTwoPromptSession<'a> {
    pub client: &'a mut AgentClient,
    pub prompts: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
    pub context: &'a HashMap<String, String>,
    pub attempt: usize,
    pub skip_repo_style: bool,
}

/// Run `reviewers_spawn`, then retry `review_write` against the same prep file.
///
/// # Errors
///
/// Returns [`WorkflowError`] when a prompt, restore, prep read, review read, or abort check fails.
pub async fn run_reviewers_spawn_then_review_write<F>(
    session: ReviewTwoPromptSession<'_>,
    max_inner_retries: usize,
    mut on_missing_retry: F,
) -> Result<ReviewWriteInnerOutcome, WorkflowError>
where
    F: FnMut(),
{
    clear_review_attempt_artifacts(session.artifacts)?;
    run_reviewers_spawn_coder_session(ReviewersSpawnCoderSession {
        client: session.client,
        prompts: session.prompts,
        artifacts: session.artifacts,
        session_dotfile_backups: session.session_dotfile_backups,
        context: session.context,
        attempt: session.attempt,
        log_attempt: session.attempt,
        skip_repo_style: session.skip_repo_style,
    })
    .await?;
    ensure_review_prep_after_reviewers_spawn(session.artifacts)?;
    let cap = max_inner_retries.max(1);
    for review_write_try in 1..=cap {
        match finish_review_write_attempt(FinishReviewWriteInput {
            client: session.client,
            prompts: session.prompts,
            artifacts: session.artifacts,
            session_dotfile_backups: session.session_dotfile_backups,
            context: session.context,
            attempt: review_write_try,
            log_attempt: session.attempt,
            skip_repo_style: session.skip_repo_style,
        })
        .await?
        {
            ReviewAttemptFinish::Lgtm => return Ok(ReviewWriteInnerOutcome::Lgtm),
            ReviewAttemptFinish::NotLgtm => return Ok(ReviewWriteInnerOutcome::NotLgtm),
            ReviewAttemptFinish::MissingArtifact => {
                if review_write_try >= cap {
                    return Ok(ReviewWriteInnerOutcome::MissingArtifactExhausted);
                }
                on_missing_retry();
            }
        }
    }
    unreachable!("inner review_write retries always return from the match")
}

#[cfg(test)]
mod tests {
    use super::{
        ReviewTwoPromptSession, ReviewWriteInnerOutcome, run_reviewers_spawn_then_review_write,
    };
    use crate::acp::{AgentClient, AgentIoOptions};
    use crate::artifacts::{
        KissConfigBackup, KissignoreBackup, MalvinChecksBackup, SessionDotfileBackups,
        create_run_artifacts_from_text,
    };
    use crate::orchestrator::workflow_context;
    use crate::prompts::PromptStore;

    #[test]
    fn review_write_inner_outcomes_are_distinct() {
        assert_ne!(
            std::mem::discriminant(&ReviewWriteInnerOutcome::Lgtm),
            std::mem::discriminant(&ReviewWriteInnerOutcome::NotLgtm)
        );
        assert_ne!(
            std::mem::discriminant(&ReviewWriteInnerOutcome::NotLgtm),
            std::mem::discriminant(&ReviewWriteInnerOutcome::MissingArtifactExhausted)
        );
    }

    #[tokio::test]
    async fn run_reviewers_spawn_then_review_write_errors_when_spawn_prompt_without_session() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let artifacts = create_run_artifacts_from_text("rwr_smoke", Some(tmp.path())).expect("art");
        let store = PromptStore::default_store();
        let ctx = workflow_context(&artifacts, &store, "code").expect("ctx");
        let mut client = AgentClient::new(
            "m".into(),
            AgentIoOptions {
                force: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        );
        let backups = SessionDotfileBackups::from_parts(
            KissConfigBackup::Missing,
            MalvinChecksBackup::Missing,
            KissignoreBackup::Missing,
        );
        let res = run_reviewers_spawn_then_review_write(
            ReviewTwoPromptSession {
                client: &mut client,
                prompts: &store,
                artifacts: &artifacts,
                session_dotfile_backups: &backups,
                context: &ctx,
                attempt: 1,
                skip_repo_style: true,
            },
            1,
            || {},
        )
        .await;
        let Err(e) = res else {
            panic!("expected reviewers_spawn without session to fail");
        };
        assert!(e.0.contains("begin_coder_session"), "unexpected: {}", e.0);
    }
}
