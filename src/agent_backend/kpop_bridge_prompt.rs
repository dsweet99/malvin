//! Prompt execution for [`super::kpop_bridge`].

use crate::acp::{AgentError, CoderPromptOptions};
use crate::artifacts::SessionDotfileBackups;

use crate::agent_backend::mini::MiniAgentClient;

pub(super) async fn run_kpop_prompt(
    client: &mut MiniAgentClient,
    prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), AgentError> {
    client
        .run_coder_prompt(
            prompt,
            log_path,
            "kpop",
            CoderPromptOptions {
                llm_phase: Some(crate::run_timing::TimingPhase::Implement),
                ..Default::default()
            },
        )
        .await
}

pub(super) async fn restore_dotfiles_or_close(
    client: &mut MiniAgentClient,
    cwd: &std::path::Path,
    session_dotfile_backups: &SessionDotfileBackups,
) -> Result<(), AgentError> {
    if let Err(e) =
        crate::acp::restore_session_dotfiles_after_success(cwd, session_dotfile_backups)
    {
        client.end_coder_session().await.ok();
        return Err(e);
    }
    Ok(())
}
