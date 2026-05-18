use std::path::{Path, PathBuf};

use crate::artifacts::SessionDotfileBackups;
use crate::orchestrator::{
    REVIEW_WRITE_MISSING_ARTIFACT_MSG, REVIEW_WRITE_MISSING_ARTIFACT_RETRY_MSG,
    ReviewTwoPromptSession, ReviewWriteInnerOutcome, fail_on_abort_for_artifacts,
    run_reviewers_spawn_then_review_write,
};
use crate::output::{MALVIN_WHO, print_stdout_line};
use crate::run_timing::TimingPhase;

use crate::cli::repo_checks::{RepoGateOutput, run_repo_workspace_gates};
use crate::cli::{LEARN_MIN_ELAPSED_MS};

use super::prep::{compose_tidy_concerns_prompt, write_checks_do_not_pass_for_artifacts};
use super::prompt::run_tidy_prompt_with_restore;
use super::{TidyAcpInput, TidyPromptRestore};

#[derive(Clone)]
pub(crate) struct TidyRecoveryPaths {
    pub work_dir: PathBuf,
    pub run_dir: PathBuf,
}

pub(crate) struct TidyRecoveryRequest<'a> {
    pub attempt: usize,
    pub max_inner_retries: usize,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
    pub paths: TidyRecoveryPaths,
}

pub(crate) enum TidyReviewAttemptOutcome {
    Lgtm,
    NotLgtm,
    MissingArtifactExhausted,
}

pub(crate) async fn tidy_review_attempt_with_retries(
    input: &mut TidyAcpInput<'_>,
    attempt: usize,
    session_dotfile_backups: &SessionDotfileBackups,
    max_inner_retries: usize,
) -> Result<TidyReviewAttemptOutcome, String> {
    let outcome = run_reviewers_spawn_then_review_write(
        ReviewTwoPromptSession {
            client: input.client,
            prompts: input.store,
            artifacts: input.artifacts,
            session_dotfile_backups,
            context: input.context,
            attempt,
            skip_repo_style: true,
        },
        max_inner_retries.max(1),
        || {
            print_stdout_line(MALVIN_WHO, REVIEW_WRITE_MISSING_ARTIFACT_RETRY_MSG);
        },
    )
    .await
    .map_err(|e| e.0)?;
    match outcome {
        ReviewWriteInnerOutcome::Lgtm => Ok(TidyReviewAttemptOutcome::Lgtm),
        ReviewWriteInnerOutcome::NotLgtm => Ok(TidyReviewAttemptOutcome::NotLgtm),
        ReviewWriteInnerOutcome::MissingArtifactExhausted => {
            Ok(TidyReviewAttemptOutcome::MissingArtifactExhausted)
        }
    }
}

pub(crate) fn tidy_fail_on_abort(
    artifacts: &crate::artifacts::RunArtifacts,
) -> Result<(), String> {
    fail_on_abort_for_artifacts(artifacts).map_err(|e| e.0)
}

pub(crate) fn tidy_learn_elapsed_threshold_ms() -> u64 {
    const ENV: &str = "MALVIN_TIDY_LEARN_MIN_ELAPSED_MS";
    std::env::var(ENV)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(LEARN_MIN_ELAPSED_MS)
}

pub(crate) fn tidy_recovery_stdout_line(log_attempt: usize, max_attempts: usize) -> String {
    format!("tidy recovery (review attempt {log_attempt}, max-loops {max_attempts})")
}

pub(crate) async fn run_tidy_concerns_coder_turn(
    input: &mut TidyAcpInput<'_>,
    attempt: usize,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    tidy_fail_on_abort(input.artifacts)?;
    print_stdout_line(MALVIN_WHO, &format!("Concerns (attempt {attempt})"));
    let concerns = compose_tidy_concerns_prompt(input.store, input.context)?;
    run_tidy_prompt_with_restore(
        input,
        TidyPromptRestore {
            prompt: concerns.as_str(),
            label: "tidy_concerns",
            phase: TimingPhase::Concerns,
            session_dotfile_backups,
            restore_context: "tidy_concerns",
        },
    )
    .await?;
    tidy_fail_on_abort(input.artifacts)
}

pub(crate) async fn run_tidy_post_concerns_recovery(
    input: &mut TidyAcpInput<'_>,
    req: TidyRecoveryRequest<'_>,
) -> Result<bool, String> {
    if run_repo_workspace_gates(
        req.paths.work_dir.as_path(),
        RepoGateOutput::Tagged,
        Some(req.paths.run_dir.as_path()),
    )
    .is_err()
    {
        return Ok(false);
    }
    let review = tidy_review_attempt_with_retries(
        input,
        req.attempt,
        req.session_dotfile_backups,
        req.max_inner_retries,
    )
    .await?;
    if matches!(review, TidyReviewAttemptOutcome::MissingArtifactExhausted) {
        return Err(format!(
            "review: {REVIEW_WRITE_MISSING_ARTIFACT_MSG} after retries"
        ));
    }
    if !matches!(review, TidyReviewAttemptOutcome::Lgtm) {
        return Ok(false);
    }
    if run_repo_workspace_gates(
        req.paths.work_dir.as_path(),
        RepoGateOutput::Tagged,
        Some(req.paths.run_dir.as_path()),
    )
    .is_ok()
    {
        tidy_fail_on_abort(input.artifacts)?;
        return Ok(true);
    }
    write_checks_do_not_pass_for_artifacts(input.artifacts)?;
    Ok(false)
}

pub(crate) async fn run_tidy_bonus_gate_recovery(
    input: &mut TidyAcpInput<'_>,
    req: TidyRecoveryRequest<'_>,
) -> Result<bool, String> {
    run_tidy_concerns_coder_turn(
        input,
        req.attempt,
        req.session_dotfile_backups,
    )
    .await?;
    run_tidy_post_concerns_recovery(input, req).await
}

pub(crate) struct TidyMaxLoopsOneRecovery<'a> {
    pub max_attempts: usize,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
    pub paths: TidyRecoveryPaths,
    pub max_inner_retries: usize,
}

pub(crate) async fn run_tidy_max_loops_one_not_lgtm_recovery(
    input: &mut TidyAcpInput<'_>,
    ctx: TidyMaxLoopsOneRecovery<'_>,
) -> Result<bool, String> {
    let recovery = ctx.max_attempts + 1;
    print_stdout_line(
        MALVIN_WHO,
        &tidy_recovery_stdout_line(recovery, ctx.max_attempts),
    );
    run_tidy_concerns_coder_turn(input, recovery, ctx.session_dotfile_backups).await?;
    let req = TidyRecoveryRequest {
        attempt: recovery,
        max_inner_retries: ctx.max_inner_retries,
        session_dotfile_backups: ctx.session_dotfile_backups,
        paths: ctx.paths,
    };
    run_tidy_post_concerns_recovery(input, req).await
}
