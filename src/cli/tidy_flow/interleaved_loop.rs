use std::borrow::Cow;

use crate::artifacts::SessionDotfileBackups;
use crate::output::{MALVIN_WHO, print_stdout_line};

use crate::repo_checks::{RepoGateOutput, run_repo_workspace_gates};

use super::prep::{compose_tidy_concerns_prompt, write_checks_do_not_pass_for_artifacts};
use super::prompt::run_tidy_prompt_with_restore;
use super::recovery::{
    run_tidy_bonus_gate_recovery,
    run_tidy_max_loops_one_not_lgtm_recovery, TidyMaxLoopsOneRecovery,
    tidy_fail_on_abort,
    tidy_recovery_stdout_line,
    tidy_review_attempt_with_retries,
    TidyRecoveryPaths,
    TidyRecoveryRequest,
    TidyReviewAttemptOutcome,
};
use super::{TidyAcpInput, TidyPromptRestore};

pub(crate) async fn run_tidy_coder_prompt_for_attempt(
    input: &mut TidyAcpInput<'_>,
    attempt: usize,
    initial_tidy_prompt: &str,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    let coder_prompt: Cow<'_, str> = if attempt == 1 {
        Cow::Borrowed(initial_tidy_prompt)
    } else {
        Cow::Owned(compose_tidy_concerns_prompt(input.store, input.context)?)
    };
    run_tidy_prompt_with_restore(
        input,
        TidyPromptRestore {
            prompt: coder_prompt.as_ref(),
            label: "tidy",
            phase: crate::run_timing::TimingPhase::Implement,
            session_dotfile_backups,
            restore_context: "tidy",
        },
    )
    .await?;
    tidy_fail_on_abort(input.artifacts)
}

pub(crate) struct TidyLgtmFinishCtx<'a, 'b> {
    input: &'a mut TidyAcpInput<'b>,
    attempt: usize,
    max_outer_iterations: usize,
    max_review_write_inner_retries: usize,
    session_dotfile_backups: &'a SessionDotfileBackups,
}

pub(crate) async fn tidy_finish_lgtm_attempt(
    ctx: TidyLgtmFinishCtx<'_, '_>,
) -> Result<Option<()>, String> {
    let paths = TidyRecoveryPaths {
        work_dir: ctx.input.artifacts.work_dir.clone(),
        run_dir: ctx.input.artifacts.run_dir.clone(),
    };
    if run_repo_workspace_gates(
        paths.work_dir.as_path(),
        RepoGateOutput::Tagged,
        Some(paths.run_dir.as_path()),
    )
    .is_ok()
    {
        tidy_fail_on_abort(ctx.input.artifacts)?;
        return Ok(Some(()));
    }
    write_checks_do_not_pass_for_artifacts(ctx.input.artifacts)?;
    if ctx.attempt < ctx.max_outer_iterations {
        return Ok(None);
    }
    let bonus = ctx.max_outer_iterations + 1;
    print_stdout_line(
        MALVIN_WHO,
        &tidy_recovery_stdout_line(bonus, ctx.max_outer_iterations),
    );
    let bonus_req = TidyRecoveryRequest {
        attempt: bonus,
        max_inner_retries: ctx.max_review_write_inner_retries,
        session_dotfile_backups: ctx.session_dotfile_backups,
        paths,
    };
    if run_tidy_bonus_gate_recovery(ctx.input, bonus_req).await? {
        tidy_fail_on_abort(ctx.input.artifacts)?;
        return Ok(Some(()));
    }
    Ok(None)
}

pub async fn run_tidy_interleaved_loop(
    input: &mut TidyAcpInput<'_>,
    initial_tidy_prompt: &str,
    session_dotfile_backups: &SessionDotfileBackups,
    max_loops: usize,
) -> Result<(), String> {
    use crate::orchestrator::REVIEW_WRITE_MISSING_ARTIFACT_MSG;
    let max_outer_iterations = crate::cli::tidy_flow::effective_tidy_max_loops(max_loops);
    let max_review_write_inner_retries = crate::cli::tidy_flow::effective_tidy_max_loops(max_loops);
    for attempt in 1..=max_outer_iterations {
        print_stdout_line(
            MALVIN_WHO,
            &format!("tidy iteration {attempt}/{max_outer_iterations}"),
        );
        run_tidy_coder_prompt_for_attempt(
            input,
            attempt,
            initial_tidy_prompt,
            session_dotfile_backups,
        )
        .await?;

        match tidy_review_attempt_with_retries(
            input,
            attempt,
            session_dotfile_backups,
            max_review_write_inner_retries,
        )
        .await?
        {
            TidyReviewAttemptOutcome::Lgtm => {
                if tidy_finish_lgtm_attempt(TidyLgtmFinishCtx {
                    input,
                    attempt,
                    max_outer_iterations,
                    max_review_write_inner_retries,
                    session_dotfile_backups,
                })
                .await?
                .is_some()
                {
                    return Ok(());
                }
            }
            TidyReviewAttemptOutcome::NotLgtm => {
                if attempt == 1 && max_outer_iterations == 1
                    && run_tidy_max_loops_one_not_lgtm_recovery(
                        input,
                        TidyMaxLoopsOneRecovery {
                            max_attempts: max_outer_iterations,
                            session_dotfile_backups,
                            paths: TidyRecoveryPaths {
                                work_dir: input.artifacts.work_dir.clone(),
                                run_dir: input.artifacts.run_dir.clone(),
                            },
                            max_inner_retries: max_review_write_inner_retries,
                        },
                    )
                    .await?
                {
                    tidy_fail_on_abort(input.artifacts)?;
                    return Ok(());
                }
            }
            TidyReviewAttemptOutcome::MissingArtifactExhausted => {
                if attempt >= max_outer_iterations {
                    return Err(format!(
                        "review: {REVIEW_WRITE_MISSING_ARTIFACT_MSG} after retries"
                    ));
                }
            }
        }
    }
    Err(format!(
        "tidy did not converge within {max_outer_iterations} iterations"
    ))
}

#[cfg(test)]
mod interleaved_loop_tests {
    use super::*;
    use crate::test_agent_client::{tidy_acp_input_parts, tidy_test_session, write_fake_gate};

    #[cfg(unix)]
    #[tokio::test]
    async fn run_tidy_coder_prompt_for_attempt_fails_without_coder_session() {
        let mut session = tidy_test_session("tidy");
        let mut input = tidy_acp_input_parts(&mut session.client, &session.artifacts, &session.store, &session.context);
        let err = run_tidy_coder_prompt_for_attempt(&mut input, 1, "tidy body", &session.backups)
            .await
            .expect_err("no session");
        assert!(!err.is_empty());
    }

    #[tokio::test]
    async fn run_tidy_interleaved_loop_errors_when_review_never_produces_artifact() {
        let mut session = tidy_test_session("tidy");
        let mut input = tidy_acp_input_parts(&mut session.client, &session.artifacts, &session.store, &session.context);
        let err = run_tidy_interleaved_loop(&mut input, "tidy prompt", &session.backups, 1)
            .await
            .expect_err("no review artifact");
        assert!(
            err.contains("tidy") || err.contains("review") || err.contains("session"),
            "got {err:?}"
        );
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn tidy_finish_lgtm_attempt_writes_checks_marker_when_gates_fail() {
        let mut session = tidy_test_session("tidy");
        let (_bin, _guard) = write_fake_gate(&session.artifacts.work_dir, "failgate", 1);
        let mut input = tidy_acp_input_parts(&mut session.client, &session.artifacts, &session.store, &session.context);
        let finished = tidy_finish_lgtm_attempt(TidyLgtmFinishCtx {
            input: &mut input,
            attempt: 1,
            max_outer_iterations: 2,
            max_review_write_inner_retries: 1,
            session_dotfile_backups: &session.backups,
        })
        .await
        .expect("finish");
        assert!(finished.is_none());
        let review =
            std::fs::read_to_string(session.artifacts.artifact_review_md()).expect("review");
        assert!(review.contains("Checks do not pass"));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn tidy_finish_lgtm_attempt_returns_some_when_gates_pass() {
        let mut session = tidy_test_session("tidy");
        let (_bin, _guard) = write_fake_gate(&session.artifacts.work_dir, "okgate", 0);
        let mut input = tidy_acp_input_parts(&mut session.client, &session.artifacts, &session.store, &session.context);
        let finished = tidy_finish_lgtm_attempt(TidyLgtmFinishCtx {
            input: &mut input,
            attempt: 1,
            max_outer_iterations: 1,
            max_review_write_inner_retries: 1,
            session_dotfile_backups: &session.backups,
        })
        .await
        .expect("finish");
        assert!(finished.is_some());
    }
}
