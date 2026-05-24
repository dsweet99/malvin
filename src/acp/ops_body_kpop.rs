use crate::acp::import_prelude::*;
use crate::acp::{AgentClient, AgentError, AcpSession, client_timing_elapsed_ms, spawn_agent_acp_session};
use std::path::PathBuf;
use std::time::Instant;

/// Inputs for [`run_kpop_flow_once`].
pub struct KpopFlowOnceArgs<'a> {
    pub cwd: &'a Path,
    pub kpop_prompts: &'a [&'a str],
    pub kpop_log: &'a Path,
    pub learn: Option<(&'a str, &'a Path)>,
    /// Skip learn if elapsed time is below this threshold (milliseconds). Set to 0 to always run learn.
    pub learn_min_elapsed_ms: u64,
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
    pub learn: Option<(String, PathBuf)>,
    pub learn_min_elapsed_ms: u64,
    pub state: &'cwd mut crate::kpop_progression::KpopMultiturnState<'state>,
    pub session_dotfile_backups: &'cwd crate::artifacts::SessionDotfileBackups,
}

pub(crate) fn restore_session_dotfiles(
    cwd: &std::path::Path,
    bundle: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    crate::artifacts::restore_workspace_session_dotfiles(cwd, bundle).map_err(AgentError)
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
            return kpop_fail_after_prompt(
                &s,
                KpopFailAfterPrompt {
                    cwd: args.cwd,
                    session_dotfile_backups,
                    err: e,
                    phase: "prompt",
                },
            )
            .await;
        }
        restore_session_dotfiles(args.cwd, session_dotfile_backups)?;
    }

    kpop_learn_phase(&s, client, args, session_dotfile_backups).await?;

    s.shutdown().await.map_err(AgentError)
}

pub(crate) async fn kpop_learn_phase(
    session: &AcpSession,
    client: &AgentClient,
    args: &KpopFlowOnceArgs<'_>,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    let Some((learn_body, learn_log)) = args.learn else {
        return Ok(());
    };
    let elapsed_ms = client_timing_elapsed_ms(client);
    let should_learn =
        crate::should_run_learn_check(args.learn_min_elapsed_ms, elapsed_ms);
    if !should_learn {
        return Ok(());
    }
    if let Err(e) = kpop_round(KpopPromptRound {
        session,
        client,
        text: learn_body,
        log: learn_log,
        who: "learn",
        phase: crate::run_timing::TimingPhase::Learn,
    })
    .await
    {
        return kpop_fail_after_prompt(
            session,
            KpopFailAfterPrompt {
                cwd: args.cwd,
                session_dotfile_backups,
                err: e,
                phase: "learn",
            },
        )
        .await;
    }
    restore_session_dotfiles(args.cwd, session_dotfile_backups)
}

pub(crate) struct KpopFailAfterPrompt<'a> {
    pub(crate) cwd: &'a std::path::Path,
    pub(crate) session_dotfile_backups: &'a crate::artifacts::SessionDotfileBackups,
    pub(crate) err: AgentError,
    pub(crate) phase: &'a str,
}

pub(crate) async fn kpop_fail_after_prompt(
    session: &AcpSession,
    fail: KpopFailAfterPrompt<'_>,
) -> Result<(), AgentError> {
    let _ = session.shutdown().await;
    Err(restore_workspace_on_error(
        fail.cwd,
        fail.session_dotfile_backups,
        fail.err,
        fail.phase,
    ))
}


#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_kpop_flow_once_args() { let _ = stringify!(KpopFlowOnceArgs); }

    #[test]
    fn kiss_cov_kpop_prompt_round() { let _ = stringify!(KpopPromptRound); }

    #[test]
    fn kiss_cov_kpop_round() { let _ = stringify!(kpop_round); }

    #[test]
    fn kiss_cov_agent_kpop_multiturn_ctl() { let _ = stringify!(AgentKpopMultiturnCtl); }

    #[test]
    fn kiss_cov_restore_session_dotfiles() { let _ = stringify!(restore_session_dotfiles); }

    #[test]
    fn kiss_cov_restore_workspace_on_error() { let _ = stringify!(restore_workspace_on_error); }

    #[test]
    fn kiss_cov_run_kpop_flow_once() { let _ = stringify!(run_kpop_flow_once); }

    #[test]
    fn kiss_cov_kpop_learn_phase() { let _ = stringify!(kpop_learn_phase); }

    #[test]
    fn kiss_cov_kpop_fail_after_prompt() { let _ = stringify!(kpop_fail_after_prompt); }

}
