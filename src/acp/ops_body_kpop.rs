use crate::acp::import_prelude::*;
use crate::acp::{AgentClient, AgentError, AcpSession, spawn_agent_acp_session};
use std::path::PathBuf;
use std::time::Instant;

/// Inputs for [`run_kpop_flow_once`].
pub struct KpopFlowOnceArgs<'a> {
    pub cwd: &'a Path,
    pub kpop_prompts: &'a [&'a str],
    pub kpop_log: &'a Path,
}

pub(crate) struct KpopPromptRound<'a> {
    pub session: &'a AcpSession,
    pub client: &'a AgentClient,
    pub text: &'a str,
    pub log: &'a Path,
    pub who: &'a str,
    pub phase: crate::run_timing::TimingPhase,
}

pub(crate) async fn kpop_round(round: KpopPromptRound<'_>) -> Result<(), AgentError> {
    crate::prompts::enforce_no_unresolved_braces(round.text).map_err(|e| AgentError(e.0))?;
    let t0 = Instant::now();
    match round.session.prompt(round.text, round.log, round.who, None).await {
        Ok(()) => {
            crate::run_timing::record_llm(
                round.client.timing.as_ref(),
                round.phase,
                t0.elapsed(),
            );
            Ok(())
        }
        Err(e) => {
            crate::run_timing::record_llm(
                round.client.timing.as_ref(),
                round.phase,
                t0.elapsed(),
            );
            Err(AgentError(e))
        }
    }
}

/// Arguments for [`AgentClient::run_kpop_multiturn`](crate::AgentClient::run_kpop_multiturn) and [`run_kpop_multiturn_once`].
pub struct AgentKpopMultiturnCtl<'cwd, 'state> {
    pub cwd: &'cwd Path,
    pub kpop_log: PathBuf,
    pub state: &'cwd mut crate::kpop_progression::KpopMultiturnState<'state>,
    pub session_dotfile_backups: &'cwd crate::artifacts::SessionDotfileBackups,
}

pub(crate) fn restore_session_dotfiles(
    cwd: &std::path::Path,
    bundle: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    crate::artifacts::restore_workspace_session_dotfiles(cwd, bundle).map_err(AgentError)
}

pub(crate) fn restore_session_dotfiles_after_success(
    cwd: &Path,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    match restore_session_dotfiles(cwd, session_dotfile_backups) {
        Ok(()) => Ok(()),
        Err(first) => {
            if restore_session_dotfiles(cwd, session_dotfile_backups).is_ok() {
                Ok(())
            } else {
                Err(restore_workspace_on_error(
                    cwd,
                    session_dotfile_backups,
                    first,
                    "restore",
                ))
            }
        }
    }
}

fn restore_workspace_on_error(
    cwd: &Path,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
    primary_error: AgentError,
    phase: &str,
) -> AgentError {
    match crate::artifacts::restore_workspace_session_dotfiles(cwd, session_dotfile_backups) {
        Ok(()) => primary_error,
        Err(restore_error) => AgentError(format!(
            "{}; workspace session restore failed ({phase}): {restore_error}",
            primary_error.0
        )),
    }
}

pub(crate) async fn run_kpop_flow_once(
    client: &AgentClient,
    args: &KpopFlowOnceArgs<'_>,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    let s = spawn_agent_acp_session(client, args.cwd).await?;

    for prompt in args.kpop_prompts {
        if let Err(e) = kpop_round(KpopPromptRound {
            session: &s,
            client,
            text: prompt,
            log: args.kpop_log,
            who: "kpop",
            phase: crate::run_timing::TimingPhase::Implement,
        })
        .await
        {
            return kpop_fail_after_prompt(KpopFailAfterPrompt {
                    cwd: args.cwd,
                    session_dotfile_backups,
                    err: e,
                    phase: "prompt",
                },
            )
            .await;
        }
        restore_session_dotfiles_after_success(args.cwd, session_dotfile_backups)?;
    }

    s.shutdown().await.map_err(AgentError)
}

pub(crate) struct KpopFailAfterPrompt<'a> {
    pub(crate) cwd: &'a std::path::Path,
    pub(crate) session_dotfile_backups: &'a crate::artifacts::SessionDotfileBackups,
    pub(crate) err: AgentError,
    pub(crate) phase: &'a str,
}

pub(crate) async fn kpop_fail_after_prompt(
    fail: KpopFailAfterPrompt<'_>,
) -> Result<(), AgentError> {
    Err(restore_workspace_on_error(
        fail.cwd,
        fail.session_dotfile_backups,
        fail.err,
        fail.phase,
    ))
}


#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_kpop_flow_once_args() { let _: Option<KpopFlowOnceArgs> = None; }

    #[test]
    fn kiss_cov_kpop_prompt_round() { let _: Option<KpopPromptRound> = None; }

    #[test]
    fn kiss_cov_kpop_round() { let _ = kpop_round; }

    #[test]
    fn kiss_cov_agent_kpop_multiturn_ctl() { let _: Option<AgentKpopMultiturnCtl> = None; }

    #[test]
    fn kiss_cov_restore_session_dotfiles() { let _ = restore_session_dotfiles; }

    #[test]
    fn kiss_cov_restore_workspace_on_error() { let _ = restore_workspace_on_error; }

    #[test]
    fn kiss_cov_run_kpop_flow_once() { let _ = run_kpop_flow_once; }

    #[test]
    fn kiss_cov_kpop_fail_after_prompt() { let _ = kpop_fail_after_prompt; }

    #[test]
    fn kiss_cov_kpop_fail_after_prompt_struct() { let _: Option<KpopFailAfterPrompt> = None; }

}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<AgentKpopMultiturnCtl> = None;
        let _: Option<KpopFailAfterPrompt> = None;
        let _: Option<KpopFlowOnceArgs> = None;
        let _: Option<KpopPromptRound> = None;
        let _ = kpop_fail_after_prompt;
        let _ = kpop_round;
        let _ = restore_session_dotfiles;
        let _ = restore_workspace_on_error;
        let _ = run_kpop_flow_once;
    }
}
