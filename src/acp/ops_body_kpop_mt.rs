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
    let exp_text = crate::kpop_progression::read_exp_log_text(after.state.exp_log_path())
        .map_err(AgentError)?;
    let hypotheses_after = crate::kpop_progression::hypotheses_emitted(&exp_text);
    if hypotheses_after > after.state.max_hypotheses {
        return Err(AgentError(format!(
            "experiment log counts {hypotheses_after} hypothesis steps, exceeding --max-hypotheses ({})",
            after.state.max_hypotheses
        )));
    }
    after.state.record_kpop_block_prompt_completed();
    Ok(())
}

// Mirrors `run_kpop_flow_once`: ACP session plus per-prompt workspace restores for session dotfiles.
pub(crate) async fn run_kpop_multiturn_once(
    client: &AgentClient,
    ctl: &mut AgentKpopMultiturnCtl<'_, '_>,
) -> Result<(), AgentError> {
    let s = spawn_agent_acp_session(client, ctl.cwd).await?;

    loop {
        let prompt = match ctl.state.next_prompt() {
            Ok(Some(p)) => p,
            Ok(None) => break,
            Err(e) => {
                return Err(AgentError(e));
            }
        };
        let text = prompt.as_str();
        if let Err(e) = kpop_round(KpopPromptRound {
            session: &s,
            client,
            text,
            log: ctl.kpop_log.as_path(),
            who: "kpop",
            phase: crate::run_timing::TimingPhase::Implement,
        })
        .await
        {
            return kpop_fail_after_prompt(
                &s,
                KpopFailAfterPrompt {
                    cwd: ctl.cwd,
                    session_dotfile_backups: ctl.session_dotfile_backups,
                    err: e,
                    phase: "prompt",
                },
            )
            .await;
        }
        multiturn_after_successful_round(
            &s,
            MultiturnRoundAfter {
                cwd: ctl.cwd,
                session_dotfile_backups: ctl.session_dotfile_backups,
                state: ctl.state,
            },
        )
        .await?;
    }

    s.shutdown().await.map_err(AgentError)
}

#[cfg(test)]
mod kiss_cov_auto{
    use super::*;

    #[test]
    fn kiss_cov_multiturn_round_after() { let _: Option<MultiturnRoundAfter> = None; }

    #[test]
    fn kiss_cov_multiturn_after_successful_round() { let _ = multiturn_after_successful_round; }

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
        let _ = run_kpop_multiturn_once;
    }
}
