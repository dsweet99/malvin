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
    })
    .await
    .map_err(AgentError)
}

/// After `session/prompt`, trace lines are already printed live when tee is on (`trace_file_write_line`).
#[allow(clippy::missing_const_for_fn)]
pub(crate) fn maybe_tee_log(_client: &AgentClient, _log_path: &Path) {}

pub(crate) async fn run_reviewer_pair_once(
    client: &AgentClient,
    pair: &ReviewerPromptPair<'_>,
    pair_id: crate::run_timing::ReviewPairId,
) -> Result<(), AgentError> {
    let s = spawn_agent_acp_session(client, pair.cwd).await?;

    let mut review_full = pair.review_body.to_string();
    if let Ok(style_text) = std::fs::read_to_string(&client.style_prompt_path) {
        let t = style_text.trim();
        if !t.is_empty() {
            review_full = format!("{t}\n\n{review_full}");
        }
    }

    let t_review = Instant::now();
    let review_out = s.prompt(&review_full, pair.review_log, pair.review_who).await;
    crate::run_timing::record_llm(
        client.timing.as_ref(),
        pair_id.review_phase(),
        t_review.elapsed(),
    );
    review_out.map_err(AgentError)?;
    maybe_tee_log(client, pair.review_log);

    // Order invariant: workspace sync → LGTM on artifact → optional kpop prompt (`review_sync::sync_review_then_is_lgtm`).
    let lgtm = sync_review_then_is_lgtm(pair.workspace_review_path, pair.artifact_review_path)
        .map_err(|e| AgentError(format!("review.md sync: {e}")))?;
    if lgtm {
        s.shutdown().await.map_err(AgentError)?;
        return Ok(());
    }

    let t_kpop = Instant::now();
    let kpop_out = s.prompt(pair.kpop_body, pair.kpop_log, pair.kpop_who).await;
    crate::run_timing::record_llm(client.timing.as_ref(), pair_id.kpop_phase(), t_kpop.elapsed());
    kpop_out.map_err(AgentError)?;
    maybe_tee_log(client, pair.kpop_log);

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
}

fn kpop_trace_sibling(base: &Path, tag: &str) -> PathBuf {
    let parent = base.parent().map_or_else(|| PathBuf::from("."), Path::to_path_buf);
    let stem = base.file_stem().and_then(|s| s.to_str()).unwrap_or("kpop");
    parent.join(format!("{stem}_{tag}.log"))
}

#[allow(clippy::too_many_lines)] // KPOP: main, optional learn, optional creative pad/roll sequence
pub(crate) async fn run_kpop_flow_once(
    client: &AgentClient,
    args: &KpopFlowOnceArgs<'_>,
) -> Result<(), AgentError> {
    /// Short continuation so the session can reach `interaction_index` 3 while respecting the “no
    /// MBC2 on the first three ACP interactions” rule (`CREATIVE_MIN_INTERACTION`).
    const KPOP_CREATIVE_CONTINUE: &str = "Continue applying the KPOP method (Hypothesize / Predict / Falsify); append new entries to an exp_log_*.md file under this run's _kpop/ directory as already instructed.";

    async fn round(
        session: &AcpSession,
        client: &AgentClient,
        text: &str,
        log: &Path,
        who: &str,
        phase: crate::run_timing::TimingPhase,
    ) -> Result<(), AgentError> {
        let t0 = Instant::now();
        match session.prompt(text, log, who).await {
            Ok(()) => {
                crate::run_timing::record_llm(client.timing.as_ref(), phase, t0.elapsed());
                maybe_tee_log(client, log);
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

    let have_learn = args.learn.is_some();

    if let Some((learn_body, learn_log)) = args.learn {
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
    }

    if crate::kpop_acp_prompt::kpop_creative_enabled(args.p_creative) {
        debug_assert_eq!(
            crate::kpop_acp_prompt::kpop_standalone_outbound_prompt_count(args.p_creative, have_learn),
            crate::kpop_acp_prompt::KPOP_SESSION_PROMPT_COUNT_WHEN_P_CREATIVE,
            "creative path must send one prompt per interaction index 0..=3"
        );
        if have_learn {
            let pad_log = kpop_trace_sibling(args.kpop_log, "creative_pad2");
            let pad_text = crate::kpop_acp_prompt::kpop_acp_user_prompt(
                &crate::kpop_acp_prompt::KpopAcpPromptPick {
                    interaction_index: 2,
                    p_creative: args.p_creative,
                    default_prompt: KPOP_CREATIVE_CONTINUE,
                    mbc2_body: args.mbc2_body,
                },
                &mut rng,
            );
            if let Err(e) = round(
                &s,
                client,
                &pad_text,
                &pad_log,
                "kpop",
                crate::run_timing::TimingPhase::Implement,
            )
            .await
            {
                let _ = s.shutdown().await;
                return Err(e);
            }
        } else {
            for (idx, tag) in [(1u32, "creative_pad1"), (2u32, "creative_pad2")] {
                let pad_log = kpop_trace_sibling(args.kpop_log, tag);
                let pad_text = crate::kpop_acp_prompt::kpop_acp_user_prompt(
                    &crate::kpop_acp_prompt::KpopAcpPromptPick {
                        interaction_index: idx,
                        p_creative: args.p_creative,
                        default_prompt: KPOP_CREATIVE_CONTINUE,
                        mbc2_body: args.mbc2_body,
                    },
                    &mut rng,
                );
                if let Err(e) = round(
                    &s,
                    client,
                    &pad_text,
                    &pad_log,
                    "kpop",
                    crate::run_timing::TimingPhase::Implement,
                )
                .await
                {
                    let _ = s.shutdown().await;
                    return Err(e);
                }
            }
        }

        let roll_log = kpop_trace_sibling(args.kpop_log, "creative_roll");
        let roll_text = crate::kpop_acp_prompt::kpop_acp_user_prompt(
            &crate::kpop_acp_prompt::KpopAcpPromptPick {
                interaction_index: 3,
                p_creative: args.p_creative,
                default_prompt: KPOP_CREATIVE_CONTINUE,
                mbc2_body: args.mbc2_body,
            },
            &mut rng,
        );
        if let Err(e) = round(
            &s,
            client,
            &roll_text,
            &roll_log,
            "kpop",
            crate::run_timing::TimingPhase::Implement,
        )
        .await
        {
            let _ = s.shutdown().await;
            return Err(e);
        }
    }

    s.shutdown().await.map_err(AgentError)
}
