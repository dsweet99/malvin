use std::collections::HashMap;
use std::path::Path;

use crate::acp::{AgentClient, AgentError, CoderPromptOptions};
use crate::artifacts::{RunArtifacts, SessionDotfileBackups, restore_workspace_session_dotfiles};
use crate::prompts::PromptStore;
use crate::run_timing::TimingPhase;

use super::constants::REVIEW_WRITE_FILE;
use super::{WorkflowError, format_prompt_path, prompt_md_stem};

pub struct ReviewWriteCoderSession<'a> {
    pub client: &'a mut AgentClient,
    pub prompts: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
    pub context: &'a HashMap<String, String>,
    pub reviewers_subdir: &'a Path,
    pub attempt: usize,
}

fn merge_coder_run_and_restore(
    run_result: Result<(), WorkflowError>,
    restore_result: Result<(), WorkflowError>,
) -> Result<(), WorkflowError> {
    match (run_result, restore_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(run_err), Ok(())) => Err(run_err),
        (Ok(()), Err(restore_err)) => Err(restore_err),
        (Err(run_err), Err(restore_err)) => {
            Err(WorkflowError(format!("{}, {}", run_err.0, restore_err.0)))
        }
    }
}

pub async fn run_review_write_coder_session(
    session: ReviewWriteCoderSession<'_>,
) -> Result<(), WorkflowError> {
    let ReviewWriteCoderSession {
        client,
        prompts,
        artifacts,
        session_dotfile_backups,
        context,
        reviewers_subdir,
        attempt,
    } = session;
    let mut write_ctx = context.clone();
    write_ctx.insert(
        "reviewers_subdir".to_string(),
        format_prompt_path(reviewers_subdir, &artifacts.work_dir),
    );
    let suffix = format!("review_write_attempt_{attempt}");
    let prompt = prompts
        .render(REVIEW_WRITE_FILE, &write_ctx)
        .map_err(|e| WorkflowError(e.0))?;
    let stem = prompt_md_stem(REVIEW_WRITE_FILE);
    let log = artifacts.log_path(&format!("coder_{stem}_{suffix}"));
    let run_result = client
        .run_coder_prompt(
            &prompt,
            &log,
            stem,
            CoderPromptOptions {
                llm_phase: Some(TimingPhase::ReviewWrite),
                skip_repo_style: false,
                do_trace_split: None,
                stdout_bracket_label: Some(REVIEW_WRITE_FILE),
            },
        )
        .await
        .map_err(|e: AgentError| WorkflowError(e.0));
    let restore_result =
        restore_workspace_session_dotfiles(&artifacts.work_dir, session_dotfile_backups)
            .map_err(WorkflowError);
    merge_coder_run_and_restore(run_result, restore_result)
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_stringify_review_fanout_write_units() {
        let _ = stringify!(super::run_review_write_coder_session);
        let _ = stringify!(super::ReviewWriteCoderSession);
        let _ = stringify!(super::merge_coder_run_and_restore);
    }
}
