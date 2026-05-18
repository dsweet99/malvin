struct MultiturnRoundAfter<'a, 'b> {
    cwd: &'a Path,
    session_dotfile_backups: &'a crate::artifacts::SessionDotfileBackups,
    state: &'a mut crate::kpop_progression::KpopMultiturnState<'b>,
    is_kpop_block: bool,
}

async fn multiturn_after_successful_round(
    session: &AcpSession,
    after: MultiturnRoundAfter<'_, '_>,
) -> Result<(), AgentError> {
    restore_session_dotfiles(after.cwd, after.session_dotfile_backups)?;
    let exp_text = crate::kpop_progression::read_exp_log_text(after.state.exp_log_path())
        .map_err(AgentError)?;
    let n = crate::kpop_progression::hypotheses_emitted(&exp_text);
    if n > after.state.max_hypotheses {
        let _ = session.shutdown().await;
        return Err(AgentError(format!(
            "experiment log counts {n} hypothesis steps, exceeding --max-hypotheses ({})",
            after.state.max_hypotheses
        )));
    }
    if after.is_kpop_block {
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
                is_kpop_block,
            },
        )
        .await?;
    }

    kpop_multiturn_learn_phase(&s, client, ctl).await?;

    s.shutdown().await.map_err(AgentError)
}

async fn kpop_multiturn_learn_phase(
    session: &AcpSession,
    client: &AgentClient,
    ctl: &AgentKpopMultiturnCtl<'_, '_>,
) -> Result<(), AgentError> {
    let Some((learn_body, learn_log)) = &ctl.learn else {
        return Ok(());
    };
    let elapsed_ms = client_timing_elapsed_ms(client);
    let should_learn = crate::should_run_learn_check(ctl.learn_min_elapsed_ms, elapsed_ms);
    if !should_learn {
        return Ok(());
    }
    if let Err(e) = kpop_round(KpopPromptRound {
        session,
        client,
        text: learn_body.as_str(),
        log: learn_log.as_path(),
        who: "learn",
        phase: crate::run_timing::TimingPhase::Learn,
    })
    .await
    {
        return kpop_fail_after_prompt(
            session,
            KpopFailAfterPrompt {
                cwd: ctl.cwd,
                session_dotfile_backups: ctl.session_dotfile_backups,
                err: e,
                phase: "learn",
            },
        )
        .await;
    }
    restore_session_dotfiles(ctl.cwd, ctl.session_dotfile_backups)
}
