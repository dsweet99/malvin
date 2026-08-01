//! K-Pop orchestration over [`AgentBackend`].

use std::path::Path;
use crate::acp::{
    backoff_after_mini_gate_failure, kpop_fail_after_prompt, retries_noun, AgentError,
    AgentKpopMultiturnCtl, CoderPromptOptions, KpopFailAfterPrompt, KpopFlowOnceArgs,
};
use crate::run_timing::TimingPhase;

use super::backend::AgentBackend;
use super::backend_ops::{agent_max_retries, agent_timing_opt};

pub(crate) async fn run_kpop_flow_via_agent(
    client: &mut AgentBackend,
    flow: &KpopFlowOnceArgs<'_>,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    crate::agent_phase::enter_kpop();
    let mut last_error = String::new();
    let mut attempts_used = 0_u32;
    let max_attempts = agent_max_retries(client);
    for attempt in 1..=max_attempts {
        attempts_used = attempt;
        match run_kpop_flow_once_via_agent(client, flow, session_dotfile_backups).await {
            Ok(()) => {
                crate::agent_phase::leave_kpop();
                return Ok(());
            }
            Err(e) => {
                last_error = e.0;
                if backoff_after_mini_gate_failure(
                    agent_timing_opt(client),
                    &last_error,
                    attempt,
                    max_attempts,
                )
                .await?
                {
                    break;
                }
            }
        }
    }
    crate::agent_phase::leave_kpop();
    let retries = attempts_used.saturating_sub(1);
    let noun = retries_noun(retries);
    Err(AgentError(format!(
        "agent (kpop flow) failed after {retries} {noun}. Last error:\n{last_error}"
    )))
}

pub(crate) async fn run_kpop_multiturn_via_agent(
    client: &mut AgentBackend,
    mut ctl: AgentKpopMultiturnCtl<'_, '_>,
) -> Result<(), AgentError> {
    crate::agent_phase::enter_kpop();
    let mut last_error = String::new();
    let mut attempts_used = 0_u32;
    let max_attempts = agent_max_retries(client);
    for attempt in 1..=max_attempts {
        attempts_used = attempt;
        match run_kpop_multiturn_once_via_agent(client, &mut ctl).await {
            Ok(()) => {
                crate::agent_phase::leave_kpop();
                return Ok(());
            }
            Err(e) => {
                ctl.state.reset_for_transport_retry();
                last_error = e.0;
                if backoff_after_mini_gate_failure(
                    agent_timing_opt(client),
                    &last_error,
                    attempt,
                    max_attempts,
                )
                .await?
                {
                    break;
                }
            }
        }
    }
    crate::agent_phase::leave_kpop();
    let retries = attempts_used.saturating_sub(1);
    let noun = retries_noun(retries);
    Err(AgentError(format!(
        "agent (kpop multiturn) failed after {retries} {noun}. Last error:\n{last_error}"
    )))
}

async fn run_kpop_flow_once_via_agent(
    client: &mut AgentBackend,
    args: &KpopFlowOnceArgs<'_>,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    begin_agent_session(client, args.cwd).await?;
    for prompt in args.kpop_prompts {
        if let Err(e) = run_agent_kpop_prompt(client, prompt, args.kpop_log).await {
            end_agent_session(client).await.ok();
            return kpop_fail_after_prompt(KpopFailAfterPrompt {
                cwd: args.cwd,
                session_dotfile_backups,
                err: e,
                phase: "prompt",
            })
            .await;
        }
        restore_dotfiles_or_close_agent(client, args.cwd, session_dotfile_backups).await?;
    }
    end_agent_session(client).await
}

async fn run_kpop_multiturn_once_via_agent(
    client: &mut AgentBackend,
    ctl: &mut AgentKpopMultiturnCtl<'_, '_>,
) -> Result<(), AgentError> {
    begin_agent_session(client, ctl.cwd).await?;
    loop {
        let prompt = match ctl.state.next_prompt() {
            Ok(Some(p)) => p,
            Ok(None) => break,
            Err(e) => {
                end_agent_session(client).await.ok();
                return Err(AgentError(e));
            }
        };
        if let Err(e) = run_agent_kpop_prompt(client, prompt.as_str(), ctl.kpop_log.as_path()).await
        {
            end_agent_session(client).await.ok();
            return kpop_fail_after_prompt(KpopFailAfterPrompt {
                cwd: ctl.cwd,
                session_dotfile_backups: ctl.session_dotfile_backups,
                err: e,
                phase: "prompt",
            })
            .await;
        }
        restore_dotfiles_or_close_agent(client, ctl.cwd, ctl.session_dotfile_backups).await?;
        ctl.state.record_kpop_block_prompt_completed();
    }
    end_agent_session(client).await
}

async fn begin_agent_session(client: &mut AgentBackend, cwd: &Path) -> Result<(), AgentError> {
    client.begin_coder_session(cwd).await
}

async fn end_agent_session(client: &mut AgentBackend) -> Result<(), AgentError> {
    client.end_coder_session().await
}

async fn run_agent_kpop_prompt(
    client: &mut AgentBackend,
    prompt: &str,
    kpop_log: &Path,
) -> Result<(), AgentError> {
    let opts = CoderPromptOptions {
        llm_phase: Some(TimingPhase::Implement),
        single_attempt: true,
        ..CoderPromptOptions::default()
    };
    client
        .run_coder_prompt(prompt, kpop_log, "kpop", opts)
        .await
}

async fn restore_dotfiles_or_close_agent(
    client: &mut AgentBackend,
    cwd: &Path,
    backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    if let Err(e) =
        crate::artifacts::restore_workspace_session_dotfiles(cwd, backups)
    {
        end_agent_session(client).await.ok();
        return Err(AgentError(e));
    }
    Ok(())
}

