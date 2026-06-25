//! Prompt execution and hypothesis-budget checks for [`super::kpop_bridge`].

use crate::acp::{AgentError, AgentKpopMultiturnCtl, CoderPromptOptions};
use crate::artifacts::SessionDotfileBackups;

use crate::agent_backend::mini::MiniAgentClient;

pub(super) fn kpop_coder_opts() -> CoderPromptOptions<'static> {
    CoderPromptOptions {
        llm_phase: Some(crate::run_timing::TimingPhase::Implement),
        ..Default::default()
    }
}

pub(super) async fn run_kpop_prompt(
    client: &mut MiniAgentClient,
    prompt: &str,
    log_path: &std::path::Path,
) -> Result<(), AgentError> {
    client
        .run_coder_prompt(prompt, log_path, "kpop", kpop_coder_opts())
        .await
}

pub(super) async fn check_hypothesis_budget(
    client: &mut MiniAgentClient,
    ctl: &AgentKpopMultiturnCtl<'_, '_>,
) -> Result<(), AgentError> {
    let exp_text = crate::kpop_progression::read_exp_log_text(ctl.state.exp_log_path())
        .map_err(AgentError)?;
    let hypotheses_after = crate::kpop_progression::hypotheses_emitted(&exp_text);
    if hypotheses_after > ctl.state.max_hypotheses {
        client.end_coder_session().await.ok();
        return Err(AgentError(format!(
            "experiment log counts {hypotheses_after} hypothesis steps, exceeding --max-hypotheses ({})",
            ctl.state.max_hypotheses
        )));
    }
    Ok(())
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
