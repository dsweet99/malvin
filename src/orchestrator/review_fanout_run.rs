use std::collections::HashMap;

use crate::acp::{AgentClient, AgentError, CoderPromptOptions};
use crate::artifacts::{RunArtifacts, SessionDotfileBackups, restore_workspace_session_dotfiles};
use crate::prompts::PromptStore;
use crate::run_timing::TimingPhase;

use super::constants::{REVIEW_WRITE_FILE, REVIEWERS_SPAWN_FILE};
use super::review_prompt_log::{ReviewPromptLog, review_prompt_log_path};
use super::workflow_merge::merge_workflow_run_and_restore;
use super::{WorkflowError, format_prompt_path, prompt_md_stem};

pub struct ReviewWriteCoderSession<'a> {
    pub client: &'a mut AgentClient,
    pub prompts: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
    pub context: &'a HashMap<String, String>,
    pub attempt: usize,
    pub log_attempt: usize,
    pub skip_repo_style: bool,
    pub stdout_bracket_label: Option<&'a str>,
}

pub struct ReviewersSpawnCoderSession<'a> {
    pub client: &'a mut AgentClient,
    pub prompts: &'a PromptStore,
    pub artifacts: &'a RunArtifacts,
    pub session_dotfile_backups: &'a SessionDotfileBackups,
    pub context: &'a HashMap<String, String>,
    pub attempt: usize,
    pub log_attempt: usize,
    pub skip_repo_style: bool,
}

struct ReviewPromptCoderSession<'a> {
    client: &'a mut AgentClient,
    prompts: &'a PromptStore,
    artifacts: &'a RunArtifacts,
    session_dotfile_backups: &'a SessionDotfileBackups,
    context: &'a HashMap<String, String>,
    prompt_file: &'static str,
    phase: TimingPhase,
    log: ReviewPromptLog,
    stdout_bracket_label: Option<&'a str>,
}

async fn run_review_prompt_coder_session(
    session: ReviewPromptCoderSession<'_>,
) -> Result<(), WorkflowError> {
    let ReviewPromptCoderSession {
        client,
        prompts,
        artifacts,
        session_dotfile_backups,
        context,
        prompt_file,
        phase,
        log,
        stdout_bracket_label,
    } = session;
    let mut write_ctx = context.clone();
    write_ctx.insert(
        "review_prep_path".to_string(),
        format_prompt_path(&artifacts.review_prep_md(), &artifacts.work_dir),
    );
    let prompt = prompts
        .render(prompt_file, &write_ctx)
        .map_err(|e| WorkflowError(e.0))?;
    let log_path = review_prompt_log_path(artifacts, log);
    let run_result = client
        .run_coder_prompt(
            &prompt,
            &log_path,
            prompt_md_stem(prompt_file),
            CoderPromptOptions {
                llm_phase: Some(phase),
                skip_repo_style: log.skip_repo_style,
                do_trace_split: None,
                stdout_bracket_label,
            },
        )
        .await
        .map_err(|e: AgentError| WorkflowError(e.0));
    let restore_result =
        restore_workspace_session_dotfiles(&artifacts.work_dir, session_dotfile_backups)
            .map_err(WorkflowError);
    merge_workflow_run_and_restore(run_result, restore_result)
}

/// # Errors
///
/// Returns [`WorkflowError`] when prompt rendering, the coder session, or restore fails.
pub async fn run_reviewers_spawn_coder_session(
    session: ReviewersSpawnCoderSession<'_>,
) -> Result<(), WorkflowError> {
    let stdout_bracket_label = if session.skip_repo_style {
        None
    } else {
        Some(REVIEWERS_SPAWN_FILE)
    };
    run_review_prompt_coder_session(ReviewPromptCoderSession {
        client: session.client,
        prompts: session.prompts,
        artifacts: session.artifacts,
        session_dotfile_backups: session.session_dotfile_backups,
        context: session.context,
        prompt_file: REVIEWERS_SPAWN_FILE,
        phase: TimingPhase::ReviewFanout,
        log: ReviewPromptLog {
            prompt_file: REVIEWERS_SPAWN_FILE,
            skip_repo_style: session.skip_repo_style,
            log_attempt: session.log_attempt,
            attempt: session.attempt,
        },
        stdout_bracket_label,
    })
    .await
}

/// # Errors
///
/// Returns [`WorkflowError`] when prompt rendering, the coder session, or restore fails.
pub async fn run_review_write_coder_session(
    session: ReviewWriteCoderSession<'_>,
) -> Result<(), WorkflowError> {
    run_review_prompt_coder_session(ReviewPromptCoderSession {
        client: session.client,
        prompts: session.prompts,
        artifacts: session.artifacts,
        session_dotfile_backups: session.session_dotfile_backups,
        context: session.context,
        prompt_file: REVIEW_WRITE_FILE,
        phase: TimingPhase::ReviewWrite,
        log: ReviewPromptLog {
            prompt_file: REVIEW_WRITE_FILE,
            skip_repo_style: session.skip_repo_style,
            log_attempt: session.log_attempt,
            attempt: session.attempt,
        },
        stdout_bracket_label: session.stdout_bracket_label,
    })
    .await
}

#[cfg(test)]
mod tests {
    #[test]
    fn kiss_stringify_review_prompt_run_units() {
        let _ = stringify!(super::run_review_write_coder_session);
        let _ = stringify!(super::run_reviewers_spawn_coder_session);
        let _ = stringify!(super::ReviewersSpawnCoderSession);
        let _ = stringify!(super::ReviewWriteCoderSession);
        let _ = stringify!(super::ReviewPromptCoderSession);
    }
}
