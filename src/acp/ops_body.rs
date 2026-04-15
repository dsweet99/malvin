pub(crate) fn resolve_agent_bin() -> Option<PathBuf> {
    std::env::var_os("MALVIN_AGENT_ACP_BIN")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(crate::env_path::agent_or_cursor_agent_bin)
}

pub(crate) fn has_api_key() -> bool {
    for key in ["CURSOR_AGENT_API_KEY", "CURSOR_API_KEY", "AGENT_API_KEY"] {
        if std::env::var_os(key).is_some_and(|v| !v.is_empty()) {
            return true;
        }
    }
    false
}

pub(crate) fn auth_probe(args: &[&str]) -> bool {
    StdCommand::new(args[0])
        .args(&args[1..])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub(crate) async fn spawn_agent_acp_session(client: &AgentClient, cwd: &Path) -> Result<AcpSession, AgentError> {
    let bin = resolve_agent_bin();
    let rpc_secs = crate::config::acp_rpc_timeout_secs_from_env();
    let model = client.model.trim();
    let model_opt = (!model.is_empty()).then_some(model);
    AcpSession::spawn(AcpSpawnArgs {
        cwd,
        bin_override: bin.as_deref(),
        api_key: None,
        auth_token: None,
        rpc_timeout: Duration::from_secs(rpc_secs),
        acp_verbose: false,
        george_acp_lane: None,
        ui_idle_notify: None,
        model: model_opt,
        force: client.io.force,
        tee_trace_stdout: !client.io.no_tee,
        raw_output: client.io.raw_output,
    })
    .await
    .map_err(AgentError)
}

/// Reviewer prompt: trace output tees live in the ACP stdout reader when enabled (`trace_file_write_line`).
pub(crate) async fn run_reviewer_pair_once(
    client: &AgentClient,
    pair: &ReviewerPromptPair<'_>,
    pair_id: crate::run_timing::ReviewPairId,
) -> Result<(), AgentError> {
    let s = spawn_agent_acp_session(client, pair.cwd).await?;

    let repo_style = read_coder_repo_style_text(&client.style_prompt_path);
    let review_full = prepend_coder_repo_style_to_prompt(pair.review_body, repo_style.as_deref());

    let t_review = Instant::now();
    let review_out = s
        .prompt(&review_full, pair.review_log, pair.review_who, None)
        .await;
    crate::run_timing::record_llm(
        client.timing.as_ref(),
        pair_id.review_phase(),
        t_review.elapsed(),
    );
    review_out.map_err(AgentError)?;

    s.shutdown().await.map_err(AgentError)?;
    Ok(())
}

/// Inputs for [`run_kpop_flow_once`] and [`AgentClient::run_kpop_flow`](crate::AgentClient::run_kpop_flow).
pub struct KpopFlowOnceArgs<'a> {
    pub cwd: &'a Path,
    pub kpop_prompt: &'a str,
    pub kpop_log: &'a Path,
    pub learn: Option<(&'a str, &'a Path)>,
    pub p_creative: f64,
    pub mbc2_body: &'a str,
    /// Skip learn if elapsed time is below this threshold (milliseconds).
    /// Uses `grounding.md` "unless short" rule. Set to 0 to always run learn.
    pub learn_min_elapsed_ms: u64,
}

pub(crate) async fn run_kpop_flow_once(
    client: &AgentClient,
    args: &KpopFlowOnceArgs<'_>,
) -> Result<(), AgentError> {
    async fn round(
        session: &AcpSession,
        client: &AgentClient,
        text: &str,
        log: &Path,
        who: &str,
        phase: crate::run_timing::TimingPhase,
    ) -> Result<(), AgentError> {
        let t0 = Instant::now();
        match session.prompt(text, log, who, None).await {
            Ok(()) => {
                crate::run_timing::record_llm(client.timing.as_ref(), phase, t0.elapsed());
                Ok(())
            }
            Err(e) => {
                crate::run_timing::record_llm(client.timing.as_ref(), phase, t0.elapsed());
                Err(AgentError(e))
            }
        }
    }

    let s = spawn_agent_acp_session(client, args.cwd).await?;

    let mut rng = StdRng::from_entropy();

    let main_prompt = crate::kpop_acp_prompt::kpop_acp_user_prompt(
        &crate::kpop_acp_prompt::KpopAcpPromptPick {
            interaction_index: 0,
            p_creative: args.p_creative,
            default_prompt: args.kpop_prompt,
            mbc2_body: args.mbc2_body,
        },
        &mut rng,
    );
    if let Err(e) = round(
        &s,
        client,
        &main_prompt,
        args.kpop_log,
        "kpop",
        crate::run_timing::TimingPhase::Implement,
    )
    .await
    {
        let _ = s.shutdown().await;
        return Err(e);
    }

    let outbound_prompts: u32 = if let Some((learn_body, learn_log)) = args.learn {
        let elapsed_ms = client.timing.as_ref().map_or(0, |t| {
            let d = t
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .elapsed_so_far();
            u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
        });
        let should_learn =
            crate::orchestrator::should_run_learn_check(args.learn_min_elapsed_ms, elapsed_ms);
        if should_learn {
            let learn_prompt = crate::kpop_acp_prompt::kpop_acp_user_prompt(
                &crate::kpop_acp_prompt::KpopAcpPromptPick {
                    interaction_index: 1,
                    p_creative: args.p_creative,
                    default_prompt: learn_body,
                    mbc2_body: args.mbc2_body,
                },
                &mut rng,
            );
            if let Err(e) = round(
                &s,
                client,
                &learn_prompt,
                learn_log,
                "learn",
                crate::run_timing::TimingPhase::Learn,
            )
            .await
            {
                let _ = s.shutdown().await;
                return Err(e);
            }
            2
        } else {
            1
        }
    } else {
        1
    };

    debug_assert!(
        outbound_prompts == 1 || outbound_prompts == 2,
        "standalone KPOP: 1 (main only) or 2 (main + learn)"
    );

    s.shutdown().await.map_err(AgentError)
}
