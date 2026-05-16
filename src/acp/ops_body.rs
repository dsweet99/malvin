pub(crate) fn resolve_agent_bin() -> Option<PathBuf> {
    std::env::var_os("MALVIN_AGENT_ACP_BIN")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(crate::support_paths::agent_or_cursor_agent_bin)
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
        .is_ok_and(|o| o.status.success())
}

pub(crate) async fn spawn_agent_acp_session(client: &AgentClient, cwd: &Path) -> Result<AcpSession, AgentError> {
    let bin = resolve_agent_bin();
    let rpc_secs = crate::support_paths::acp_rpc_timeout_secs_from_env();
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
        show_thoughts_on_stdout: client.io.show_thoughts_on_stdout,
        emit_stdout_markdown: client.io.emit_stdout_markdown,
        prompts_log_run_dir: client.prompts_log_run_dir.as_deref(),
        log_full_outgoing_prompts: client.io.log_full_outgoing_prompts,
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
    crate::prompts::enforce_no_unresolved_braces(&review_full).map_err(|e| AgentError(e.0))?;

    let t_review = Instant::now();
    let review_out = s
        .prompt(&review_full, pair.review_log, pair.review_who, None)
        .await;
    crate::run_timing::record_llm(
        client.timing.as_ref(),
        pair_id.review_phase(),
        t_review.elapsed(),
    );
    if let Err(e) = review_out {
        let _ = s.shutdown().await;
        return Err(AgentError(e));
    }

    if pair.sync_workspace_review {
        let Some(artifact_review_path) = pair.artifact_review_path else {
            return Err(AgentError(
                "artifact_review_path required when sync_workspace_review is true".to_string(),
            ));
        };
        sync_review_to_artifact(pair.workspace_review_path, artifact_review_path)?;
    }

    s.shutdown().await.map_err(AgentError)?;
    Ok(())
}

fn sync_review_to_artifact(
    workspace_review_path: &std::path::Path,
    artifact_review_path: &std::path::Path,
) -> Result<(), AgentError> {
    let workspace_text = match std::fs::read_to_string(workspace_review_path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => {
            return Err(AgentError(format!(
                "failed to read workspace review path {}: {e}",
                workspace_review_path.display()
            )))
        }
    };

    if workspace_text.trim().is_empty() {
        return Ok(());
    }

    std::fs::write(artifact_review_path, &workspace_text).map_err(|e| {
        AgentError(format!(
            "failed to write artifact review path {}: {e}",
            artifact_review_path.display()
        ))
    })?;
    Ok(())
}

/// Inputs for [`run_kpop_flow_once`].
pub struct KpopFlowOnceArgs<'a> {
    pub cwd: &'a Path,
    pub kpop_prompts: &'a [&'a str],
    pub kpop_log: &'a Path,
    pub learn: Option<(&'a str, &'a Path)>,
    /// Skip learn if elapsed time is below this threshold (milliseconds). Set to 0 to always run learn.
    pub learn_min_elapsed_ms: u64,
}

async fn kpop_round(
    session: &AcpSession,
    client: &AgentClient,
    text: &str,
    log: &Path,
    who: &str,
    phase: crate::run_timing::TimingPhase,
) -> Result<(), AgentError> {
    crate::prompts::enforce_no_unresolved_braces(text).map_err(|e| AgentError(e.0))?;
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
        if let Err(e) = kpop_round(
            &s,
            client,
            prompt,
            args.kpop_log,
            "kpop",
            crate::run_timing::TimingPhase::Implement,
        )
        .await
        {
            let _ = s.shutdown().await;
            return Err(restore_workspace_on_error(
                args.cwd,
                session_dotfile_backups,
                e,
                "prompt",
            ));
        }
        crate::artifacts::restore_workspace_session_dotfiles(args.cwd, session_dotfile_backups)
            .map_err(AgentError)?;
    }

    if let Some((learn_body, learn_log)) = args.learn {
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
            if let Err(e) = kpop_round(
                &s,
                client,
                learn_body,
                learn_log,
                "learn",
                crate::run_timing::TimingPhase::Learn,
            )
            .await
            {
                let _ = s.shutdown().await;
                return Err(restore_workspace_on_error(
                    args.cwd,
                    session_dotfile_backups,
                    e,
                    "learn",
                ));
            }
            crate::artifacts::restore_workspace_session_dotfiles(args.cwd, session_dotfile_backups)
                .map_err(AgentError)?;
        }
    }

    s.shutdown().await.map_err(AgentError)
}

#[cfg(test)]
mod ops_body_tests {
    use super::sync_review_to_artifact;

    #[test]
    fn sync_review_to_artifact_copies_workspace_review_content() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let workspace_review = tmp.path().join("review.md");
        let artifact_review = tmp.path().join("run").join("review.md");
        std::fs::create_dir_all(
            artifact_review
                .parent()
                .expect("artifact parent"),
        )
        .expect("artifact parent");

        std::fs::write(&workspace_review, "LGTM\n").expect("write workspace review");

        sync_review_to_artifact(&workspace_review, &artifact_review).expect("sync");
        let synced = std::fs::read_to_string(&artifact_review).expect("artifact read");
        assert_eq!(synced, "LGTM\n");
    }
}

// Mirrors `run_kpop_flow_once`: ACP session plus per-prompt workspace restores for session dotfiles.
#[allow(clippy::too_many_arguments)]
pub(crate) async fn run_kpop_multiturn_once<B: crate::kpop_multiturn_prompts::KpopMultiturnPrompts>(
    client: &AgentClient,
    cwd: &std::path::Path,
    kpop_log: &std::path::Path,
    learn: Option<(&str, &std::path::Path)>,
    learn_min_elapsed_ms: u64,
    state: &mut crate::kpop_progression::KpopMultiturnState<B>,
    session_dotfile_backups: &crate::artifacts::SessionDotfileBackups,
) -> Result<(), AgentError> {
    let s = spawn_agent_acp_session(client, cwd).await?;

    loop {
        let prompt = match state.next_prompt() {
            Ok(Some(p)) => p,
            Ok(None) => break,
            Err(e) => {
                let _ = s.shutdown().await;
                return Err(AgentError(e));
            }
        };
        let is_kpop_block = matches!(prompt, crate::multiturn_prompt::MultiturnPrompt::KpopBlock(_));
        let text = prompt.as_str();
        if let Err(e) = kpop_round(
            &s,
            client,
            text,
            kpop_log,
            "kpop",
            crate::run_timing::TimingPhase::Implement,
        )
        .await
        {
            let _ = s.shutdown().await;
            return Err(restore_workspace_on_error(
                cwd,
                session_dotfile_backups,
                e,
                "prompt",
            ));
        }
        crate::artifacts::restore_workspace_session_dotfiles(cwd, session_dotfile_backups)
            .map_err(AgentError)?;
        let exp_text = crate::kpop_progression::read_exp_log_text(state.exp_log_path())
            .map_err(AgentError)?;
        let n = crate::kpop_progression::hypotheses_emitted(&exp_text);
        if n > state.max_hypotheses {
            let _ = s.shutdown().await;
            return Err(AgentError(format!(
                "experiment log counts {n} hypothesis steps, exceeding --max-hypotheses ({})",
                state.max_hypotheses
            )));
        }
        if is_kpop_block {
            state.record_kpop_block_prompt_completed();
        } else {
            state.record_mbc2_prompt_completed();
        }
    }

    if let Some((learn_body, learn_log)) = learn {
        let elapsed_ms = client.timing.as_ref().map_or(0, |t| {
            let d = t
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .elapsed_so_far();
            u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
        });
        let should_learn =
            crate::orchestrator::should_run_learn_check(learn_min_elapsed_ms, elapsed_ms);
        if should_learn {
            if let Err(e) = kpop_round(
                &s,
                client,
                learn_body,
                learn_log,
                "learn",
                crate::run_timing::TimingPhase::Learn,
            )
            .await
            {
                let _ = s.shutdown().await;
                return Err(restore_workspace_on_error(
                    cwd,
                    session_dotfile_backups,
                    e,
                    "learn",
                ));
            }
            crate::artifacts::restore_workspace_session_dotfiles(cwd, session_dotfile_backups)
                .map_err(AgentError)?;
        }
    }

    s.shutdown().await.map_err(AgentError)
}
