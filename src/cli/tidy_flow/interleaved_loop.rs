use crate::artifacts::SessionDotfileBackups;
use crate::output::{MALVIN_WHO, print_stdout_line};

use super::interleaved_lgtm::{run_tidy_coder_prompt_for_attempt, tidy_finish_lgtm_attempt, TidyLgtmFinishCtx};
use super::recovery::{
    run_tidy_max_loops_one_not_lgtm_recovery, tidy_fail_on_abort, tidy_review_attempt_with_retries,
    TidyMaxLoopsOneRecovery, TidyRecoveryPaths, TidyReviewAttemptOutcome,
};
use super::TidyAcpInput;

pub struct TidyInterleavedLoopOpts<'a> {
    pub session_dotfile_backups: &'a SessionDotfileBackups,
    pub max_loops: usize,
    pub quick: bool,
}

pub async fn run_tidy_interleaved_loop(
    input: &mut TidyAcpInput<'_>,
    initial_tidy_prompt: &str,
    opts: TidyInterleavedLoopOpts<'_>,
) -> Result<(), String> {
    use crate::orchestrator::REVIEW_WRITE_MISSING_ARTIFACT_MSG;
    let TidyInterleavedLoopOpts {
        session_dotfile_backups,
        max_loops,
        quick,
    } = opts;
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

        if quick {
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
            continue;
        }

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
mod tests {
    use super::*;
    use crate::test_agent_client::{tidy_acp_input_parts, tidy_test_session};

    #[tokio::test]
    async fn run_tidy_interleaved_loop_errors_when_review_never_produces_artifact() {
        let mut session = tidy_test_session("tidy");
        let mut input = tidy_acp_input_parts(
            &mut session.client,
            &session.artifacts,
            &session.store,
            &session.context,
        );
        let err = run_tidy_interleaved_loop(
            &mut input,
            "tidy prompt",
            TidyInterleavedLoopOpts {
                session_dotfile_backups: &session.backups,
                max_loops: 1,
                quick: false,
            },
        )
        .await
        .expect_err("no review artifact");
        assert!(
            err.contains("tidy") || err.contains("review") || err.contains("session"),
            "got {err:?}"
        );
    }
}
