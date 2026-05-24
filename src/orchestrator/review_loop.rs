use std::collections::HashMap;

use crate::repo_checks::RepoGateFailure;

use super::pre_review_gates::{
    run_pre_review_workspace_gates, write_pre_review_gate_failure_for_artifacts,
};
use super::review_attempt_kernel::{
    REVIEW_WRITE_INNER_RETRY_CAP, REVIEW_WRITE_MISSING_ARTIFACT_MSG,
    REVIEW_WRITE_MISSING_ARTIFACT_RETRY_MSG, clear_review_attempt_artifacts,
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
    (orchestrator.progress_callback)(&format!("Pre-review (attempt {})", ctx.attempt));
    clear_review_attempt_artifacts(orchestrator.artifacts)?;
    match run_pre_review_workspace_gates(orchestrator.artifacts) {
        Ok(()) => {}
        Err(RepoGateFailure::Command(failure)) => {
            let log_path = ctx
                .context
                .get("quality_gates_log")
                .map_or("./_malvin/.../quality_gates.log", String::as_str);
            write_pre_review_gate_failure_for_artifacts(
                orchestrator.artifacts,
                &failure,
                log_path,
            )?;
            let concern_suffix = format!("pre_review_gates_attempt_{}", ctx.attempt);
            run_concerns_and_check_abort_impl(
                orchestrator,
                ctx.attempt,
                &concern_suffix,
                ctx.context,
            )
            .await?;
            return Ok(CodeReviewAttemptOutcome::NotLgtm);
        }
        Err(RepoGateFailure::Message(message)) => return Err(WorkflowError(message)),
    }

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
    use crate::acp::{AgentClient, AgentIoOptions};
    use crate::artifacts::{
        KissConfigBackup, KissignoreBackup, MalvinChecksBackup, SessionDotfileBackups,
        create_run_artifacts_from_text,
    };
    use crate::orchestrator::{Orchestrator, WorkflowConfig, workflow_context};
    use crate::prompts::PromptStore;

    use super::run_code_review_phase;

    #[tokio::test]
    async fn run_code_review_phase_spawn_fails() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let store = PromptStore::default_store();
        let artifacts = create_run_artifacts_from_text("rv", Some(tmp.path())).expect("art");
        let ctx = workflow_context(&artifacts, &store, "code").expect("ctx");
        let mut client = AgentClient::new(
            "m".into(),
            AgentIoOptions {
                force: false,
                sandbox: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        );
        let mut orch = Orchestrator {
            client: &mut client,
            prompts: &store,
            artifacts: &artifacts,
            config: WorkflowConfig {
                max_loops: 1,
                run_learn: false,
                learn_min_elapsed_ms: 0,
                skip_check_plan: true,
            },
            progress_callback: Box::new(|_| {}),
            session_dotfile_backups: SessionDotfileBackups::from_parts(
                KissConfigBackup::Missing,
                MalvinChecksBackup::Missing,
                KissignoreBackup::Missing,
            ),
        };
        let err = run_code_review_phase(&mut orch, &ctx)
            .await
            .expect_err("review");
        assert!(!err.0.is_empty());
    }

    #[tokio::test]
    async fn code_review_single_attempt_errors_when_spawn_fails() {
        use super::{CodeReviewAttempt, CodeReviewAttemptOutcome, code_review_single_attempt};

        let tmp = tempfile::tempdir().expect("tempdir");
        let store = PromptStore::default_store();
        let artifacts = create_run_artifacts_from_text("rv-single", Some(tmp.path())).expect("art");
        let ctx = workflow_context(&artifacts, &store, "code").expect("ctx");
        let mut client = AgentClient::new(
            "m".into(),
            AgentIoOptions {
                force: false,
                sandbox: false,
                no_tee: true,
                raw_output: true,
                show_thoughts_on_stdout: false,
                emit_stdout_markdown: false,
                log_full_outgoing_prompts: false,
            },
        );
        let mut orch = Orchestrator {
            client: &mut client,
            prompts: &store,
            artifacts: &artifacts,
            config: WorkflowConfig {
                max_loops: 1,
                run_learn: false,
                learn_min_elapsed_ms: 0,
                skip_check_plan: true,
            },
            progress_callback: Box::new(|_| {}),
            session_dotfile_backups: SessionDotfileBackups::from_parts(
                KissConfigBackup::Missing,
                MalvinChecksBackup::Missing,
                KissignoreBackup::Missing,
            ),
        };
        let attempt_ctx = CodeReviewAttempt {
            context: &ctx,
            attempt: 1,
        };
        match code_review_single_attempt(&mut orch, attempt_ctx).await {
            Err(e) => assert!(!e.0.is_empty()),
            Ok(CodeReviewAttemptOutcome::Lgtm) => panic!("expected spawn failure"),
            Ok(CodeReviewAttemptOutcome::NotLgtm) => panic!("expected spawn failure"),
            Ok(CodeReviewAttemptOutcome::MissingArtifactReview) => {
                panic!("expected spawn failure")
            }
        }
    }
}
