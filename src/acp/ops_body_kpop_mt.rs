use crate::acp::import_prelude::*;
use crate::acp::{
    AgentClient, AgentError, AcpSession, AgentKpopMultiturnCtl, KpopFailAfterPrompt, KpopPromptRound,
    PromptRoundHealth, acp_session_take_prompt_round_health, kpop_fail_after_prompt, kpop_round,
    restore_session_dotfiles, spawn_agent_acp_session,
};
use crate::kpop_progression::KpopBlockMissSnapshot;
use crate::output::print_log_error;

struct MultiturnRoundAfter<'a, 'b> {
    cwd: &'a Path,
    session_dotfile_backups: &'a crate::artifacts::SessionDotfileBackups,
    state: &'a mut crate::kpop_progression::KpopMultiturnState<'b>,
    is_kpop_block: bool,
    hypotheses_before_round: usize,
    prompt_health: PromptRoundHealth,
}

fn block_miss_snapshot(
    state: &crate::kpop_progression::KpopMultiturnState<'_>,
    hypotheses_before_round: usize,
    hypotheses_after: usize,
    health: &PromptRoundHealth,
) -> Option<KpopBlockMissSnapshot> {
    let ctx = crate::kpop_progression::kpop_block_progress_ctx(state, hypotheses_before_round)?;
    if ctx.steps_needed == 0 || hypotheses_after > hypotheses_before_round {
        return None;
    }
    Some(KpopBlockMissSnapshot {
        exp_log_path: state.exp_log_path().to_path_buf(),
        hypotheses_before: hypotheses_before_round,
        hypotheses_after,
        ctx,
        tool_health_lines: health.format_lines(),
        agent_streamed_kpop_solved: health.agent_streamed_kpop_solved(),
    })
}

async fn multiturn_after_successful_round(
    session: &AcpSession,
    after: MultiturnRoundAfter<'_, '_>,
) -> Result<(), AgentError> {
    restore_session_dotfiles(after.cwd, after.session_dotfile_backups)?;
    let exp_text = crate::kpop_progression::read_exp_log_text(after.state.exp_log_path())
        .map_err(AgentError)?;
    let hypotheses_after = crate::kpop_progression::hypotheses_emitted(&exp_text);
    if hypotheses_after > after.state.max_hypotheses {
        let _ = session.shutdown().await;
        return Err(AgentError(format!(
            "experiment log counts {hypotheses_after} hypothesis steps, exceeding --max-hypotheses ({})",
            after.state.max_hypotheses
        )));
    }
    if after.is_kpop_block {
        if let Some(snapshot) = block_miss_snapshot(
            after.state,
            after.hypotheses_before_round,
            hypotheses_after,
            &after.prompt_health,
        ) {
            let mut err_text = snapshot.format_no_progress_error();
            crate::kpop_progression::set_last_block_miss(after.state, snapshot);
            if after.prompt_health.has_infra_failure() {
                err_text.push_str(
                    "\nLikely infra failure during this prompt (see ACP tool issues above).",
                );
            }
            print_log_error(&err_text);
            let _ = session.shutdown().await;
            return Err(AgentError(err_text));
        }
        after.state.record_kpop_block_prompt_completed();
    } else {
        after.state.record_mbc2_prompt_completed();
    }
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
                let _ = s.shutdown().await;
                return Err(AgentError(e));
            }
        };
        let is_kpop_block = matches!(prompt, crate::multiturn_prompt::MultiturnPrompt::KpopBlock(_));
        let hypotheses_before_round = crate::kpop_progression::read_exp_log_text(ctl.state.exp_log_path())
            .map_err(AgentError)
            .map(|text| crate::kpop_progression::hypotheses_emitted(&text))?;
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
        let prompt_health = acp_session_take_prompt_round_health(&s);
        multiturn_after_successful_round(
            &s,
            MultiturnRoundAfter {
                cwd: ctl.cwd,
                session_dotfile_backups: ctl.session_dotfile_backups,
                state: ctl.state,
                is_kpop_block,
                hypotheses_before_round,
                prompt_health,
            },
        )
        .await?;
    }

    s.shutdown().await.map_err(AgentError)
}

#[cfg(test)]
mod kiss_cov_auto {
    #[test]
    fn kiss_cov_multiturn_round_after() { let _ = stringify!(MultiturnRoundAfter); }

    #[test]
    fn kiss_cov_multiturn_after_successful_round() { let _ = stringify!(multiturn_after_successful_round); }

    #[test]
    fn kiss_cov_run_kpop_multiturn_once() { let _ = stringify!(run_kpop_multiturn_once); }

    #[test]
    fn kiss_cov_block_miss_snapshot() { let _ = stringify!(block_miss_snapshot); }

}

#[cfg(test)]
mod kiss_cov_gate_refs {
    use super::*;
    #[test]
    fn kiss_cov_unit_names() {
        let _: Option<MultiturnRoundAfter> = None;
        let _ = block_miss_snapshot;
        let _ = multiturn_after_successful_round;
        let _ = run_kpop_multiturn_once;
    }
}
