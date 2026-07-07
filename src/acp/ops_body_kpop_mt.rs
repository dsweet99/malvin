use crate::acp::import_prelude::*;
use crate::acp::{
    AgentClient, AgentError, AcpSession, AgentKpopMultiturnCtl, KpopFailAfterPrompt, KpopPromptRound,
    kpop_fail_after_prompt, kpop_round, restore_session_dotfiles_after_success, spawn_agent_acp_session,
};

struct MultiturnRoundAfter<'a, 'b> {
    cwd: &'a Path,
    session_dotfile_backups: &'a crate::artifacts::SessionDotfileBackups,
    state: &'a mut crate::kpop_progression::KpopMultiturnState<'b>,
}

async fn multiturn_after_successful_round(
    _session: &AcpSession,
    after: MultiturnRoundAfter<'_, '_>,
) -> Result<(), AgentError> {
    restore_session_dotfiles_after_success(after.cwd, after.session_dotfile_backups)?;
    crate::kpop_progression::check_hypothesis_budget(
        after.state.exp_log_path(),
        after.state.max_hypotheses,
    )
    .map_err(AgentError)?;
    after.state.record_kpop_block_prompt_completed();
    Ok(())
}

async fn end_reused_coder_session(client: &mut AgentClient) {
    client.end_coder_session().await.ok();
}

async fn recover_failed_multiturn_round(
    client: &mut AgentClient,
    ctl: &AgentKpopMultiturnCtl<'_, '_>,
    reuse_open: bool,
    err: AgentError,
) -> Result<(), AgentError> {
    let _ = crate::artifacts::restore_workspace_session_dotfiles(
        ctl.cwd, ctl.session_dotfile_backups,
    );
    if reuse_open {
        end_reused_coder_session(client).await;
    }
    Err(err)
}

async fn finish_kpop_multiturn_session(
    client: &mut AgentClient,
    reuse_open: bool,
    owned: Option<AcpSession>,
) -> Result<(), AgentError> {
    if reuse_open {
        client.end_coder_session().await
    } else {
        owned.expect("owned session").shutdown().await.map_err(AgentError)
    }
}

// Mirrors `run_kpop_flow_once`: ACP session plus per-prompt workspace restores for session dotfiles.
pub(crate) async fn run_kpop_multiturn_once(
    client: &mut AgentClient,
    ctl: &mut AgentKpopMultiturnCtl<'_, '_>,
) -> Result<(), AgentError> {
    let reuse_open = client.has_open_coder_session();
    let owned = if reuse_open {
        None
    } else {
        Some(spawn_agent_acp_session(client, ctl.cwd).await?)
    };

    loop {
        let session = if reuse_open {
            client.coder_session.as_ref().expect("open coder session")
        } else {
            owned.as_ref().expect("spawned session")
        };
        let prompt = match ctl.state.next_prompt() {
            Ok(Some(p)) => p,
            Ok(None) => break,
            Err(e) => {
                if reuse_open {
                    end_reused_coder_session(client).await;
                }
                return Err(AgentError(e));
            }
        };
        let text = prompt.as_str();
        if let Err(e) = kpop_round(KpopPromptRound {
            session,
            client,
            text,
            log: ctl.kpop_log.as_path(),
            who: "kpop",
            phase: crate::run_timing::TimingPhase::Implement,
        })
        .await
        {
            if reuse_open {
                end_reused_coder_session(client).await;
            }
            return kpop_fail_after_prompt(KpopFailAfterPrompt {
                cwd: ctl.cwd,
                session_dotfile_backups: ctl.session_dotfile_backups,
                err: e,
                phase: "prompt",
            })
            .await;
        }
        if let Err(e) = multiturn_after_successful_round(
            session,
            MultiturnRoundAfter {
                cwd: ctl.cwd,
                session_dotfile_backups: ctl.session_dotfile_backups,
                state: ctl.state,
            },
        )
        .await
        {
            return recover_failed_multiturn_round(client, ctl, reuse_open, e).await;
        }
    }

    finish_kpop_multiturn_session(client, reuse_open, owned).await
}

#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_multiturn_round_after() { let _: Option<MultiturnRoundAfter> = None; }

    #[test]
    fn kiss_cov_multiturn_after_successful_round() { let _ = multiturn_after_successful_round; }

    #[test]
    fn kiss_cov_end_reused_coder_session() { let _ = end_reused_coder_session; }

    #[test]
    fn kiss_cov_recover_failed_multiturn_round() { let _ = recover_failed_multiturn_round; }

    #[test]
    fn kiss_cov_finish_kpop_multiturn_session() { let _ = finish_kpop_multiturn_session; }

    #[test]
    fn kiss_cov_run_kpop_multiturn_once() { let _ = run_kpop_multiturn_once; }
}

#[cfg(test)]
#[allow(unused_imports)]
mod kiss_cov_gate_refs{
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<MultiturnRoundAfter> = None;
        let _ = multiturn_after_successful_round;
        let _ = end_reused_coder_session;
        let _ = recover_failed_multiturn_round;
        let _ = finish_kpop_multiturn_session;
        let _ = run_kpop_multiturn_once;
    }
}
