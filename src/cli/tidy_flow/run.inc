use std::sync::{Arc, Mutex};

use crate::artifacts::{RunArtifacts, SessionDotfileBackups};
use crate::prompts::{HEADER_MD, PromptError};
use crate::run_timing::{RunTiming, TimingPhase};

use crate::repo_checks::{RepoGateOutput, run_repo_workspace_gates};
use super::prompt::run_tidy_prompt_with_restore;
use super::recovery::{tidy_fail_on_abort, tidy_learn_elapsed_threshold_ms};
use super::interleaved_loop::run_tidy_interleaved_loop;
use super::{TidyAcpInput, TidyPromptRestore};

async fn run_tidy_learn_prompt_if_elapsed(
    input: &mut TidyAcpInput<'_>,
    timing: &Arc<Mutex<RunTiming>>,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    use crate::orchestrator::should_run_learn_check;
    if !input.run_learn {
        return Ok(());
    }
    let elapsed_ms = timing
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .elapsed_so_far()
        .as_millis();
    if !should_run_learn_check(
        tidy_learn_elapsed_threshold_ms(),
        u64::try_from(elapsed_ms).unwrap_or(u64::MAX),
    ) {
        return Ok(());
    }
    let learn_prompt = input
        .store
        .render("learn.md", input.context)
        .map_err(|e: PromptError| e.0)?;
    run_tidy_prompt_with_restore(
        input,
        TidyPromptRestore {
            prompt: &learn_prompt,
            label: "learn",
            phase: TimingPhase::Learn,
            session_dotfile_backups,
            restore_context: "learn",
        },
    )
    .await?;
    tidy_fail_on_abort(input.artifacts)
}

async fn run_tidy_summary_prompt(
    input: &mut TidyAcpInput<'_>,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    let header_only = input
        .store
        .render_prompt_only(HEADER_MD, input.context)
        .map_err(|e: PromptError| e.0)?;
    let summary_only = input
        .store
        .render("summary.md", input.context)
        .map_err(|e: PromptError| e.0)?;
    let summary_prompt = format!(
        "{}\n\n{}",
        header_only.trim_end(),
        summary_only.trim_end()
    );
    run_tidy_prompt_with_restore(
        input,
        TidyPromptRestore {
            prompt: &summary_prompt,
            label: "summary",
            phase: TimingPhase::Summary,
            session_dotfile_backups,
            restore_context: "summary",
        },
    )
    .await
}

async fn run_tidy_learn_mid_gates_and_summary(
    input: &mut TidyAcpInput<'_>,
    timing: &Arc<Mutex<RunTiming>>,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    run_tidy_learn_prompt_if_elapsed(input, timing, session_dotfile_backups).await?;
    tidy_fail_on_abort(input.artifacts)?;
    run_repo_workspace_gates(
        &input.artifacts.work_dir,
        RepoGateOutput::Tagged,
        Some(&input.artifacts.run_dir),
    )?;
    run_tidy_summary_prompt(input, session_dotfile_backups).await?;
    Ok(())
}

pub async fn run_tidy_acp(
    input: &mut TidyAcpInput<'_>,
    prompt: &str,
    session_dotfile_backups: &SessionDotfileBackups,
    max_loops: usize,
) -> Result<(), String> {
    let timing = input.client.attach_run_timing_for_session();
    input.client.prompts_log_run_dir = Some(input.artifacts.run_dir.clone());
    let begin_res = input
        .client
        .begin_coder_session(&input.artifacts.work_dir)
        .await;
    if let Err(e) = begin_res {
        input.client.set_run_timing(None);
        return Err(e.to_string());
    }

    let mut acp_result = run_tidy_interleaved_loop(
        input,
        prompt,
        session_dotfile_backups,
        max_loops,
    )
    .await;
    if acp_result.is_ok() {
        acp_result =
            run_tidy_learn_mid_gates_and_summary(input, &timing, session_dotfile_backups).await;
    }
    let end_result = input
        .client
        .end_coder_session()
        .await
        .map_err(|e| e.to_string());
    if end_result.is_err() {
        if acp_result.is_ok() {
            acp_result = end_result;
        } else {
            acp_result = Err(format!("{acp_result:?} end_coder_session: {end_result:?}"));
        }
    }

    crate::acp_post_run::emit_run_timing_after_acp(
        input.client,
        &input.artifacts.run_dir,
        &timing,
        acp_result,
    )
}

pub fn merge_tidy_timing(
    result: Result<(), String>,
    artifacts: &RunArtifacts,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), String> {
    crate::acp_post_run::merge_acp_with_workspace_session_restore_and_check_abort(
        result,
        &artifacts.work_dir,
        session_dotfile_backups,
        &artifacts.artifact_result_md(),
    )?;
    Ok(())
}
