use malvin::acp::CoderPromptOptions;
use malvin::artifacts::restore_workspace_session_dotfiles;
use malvin::run_timing::TimingPhase;

use super::TidyAcpInput;
use super::TidyPromptRestore;

pub async fn run_tidy_prompt(
    input: &mut TidyAcpInput<'_>,
    prompt: &str,
    kind: &str,
    phase: TimingPhase,
) -> Result<(), String> {
    input
        .client
        .run_coder_prompt(
            prompt,
            &input.artifacts.log_path(kind),
            kind,
            CoderPromptOptions {
                llm_phase: Some(phase),
                skip_repo_style: true,
                do_trace_split: None,
                stdout_bracket_label: None,
            },
        )
        .await
        .map_err(|e| e.to_string())
}

pub async fn run_tidy_prompt_with_restore(
    input: &mut TidyAcpInput<'_>,
    request: TidyPromptRestore<'_>,
) -> Result<(), String> {
    let acp_result = run_tidy_prompt(input, request.prompt, request.label, request.phase).await;
    let restore_result = restore_workspace_session_dotfiles(
        &input.artifacts.work_dir,
        request.session_dotfile_backups,
    )
    .map_err(|e| format!("tidy restore failed after {}: {e}", request.restore_context));
    match (acp_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(e), Ok(())) | (Ok(()), Err(e)) => Err(e),
        (Err(e), Err(restore_error)) => Err(format!(
            "{e}; tidy restore failed after {}: {restore_error}",
            request.restore_context
        )),
    }
}
