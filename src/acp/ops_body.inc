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
) -> Result<(), AgentError> {
    let s = spawn_agent_acp_session(client, pair.cwd).await?;

    let mut review_full = pair.review_body.to_string();
    if let Ok(style_text) = std::fs::read_to_string(&client.style_prompt_path) {
        let t = style_text.trim();
        if !t.is_empty() {
            review_full = format!("{t}\n\n{review_full}");
        }
    }

    s.prompt(&review_full, pair.review_log)
        .await
        .map_err(AgentError)?;
    maybe_tee_log(client, pair.review_log);

    sync_review_file(pair.workspace_review_path, pair.artifact_review_path);
    if is_lgtm(pair.artifact_review_path) {
        s.shutdown().await.map_err(AgentError)?;
        return Ok(());
    }

    s.prompt(pair.kpop_body, pair.kpop_log)
        .await
        .map_err(AgentError)?;
    maybe_tee_log(client, pair.kpop_log);

    s.shutdown().await.map_err(AgentError)?;
    Ok(())
}

pub(crate) struct KpopFlowOnceArgs<'a> {
    pub cwd: &'a Path,
    pub kpop_prompt: &'a str,
    pub kpop_log: &'a Path,
    pub learn: Option<(&'a str, &'a Path)>,
}

pub(crate) async fn run_kpop_flow_once(
    client: &AgentClient,
    args: KpopFlowOnceArgs<'_>,
) -> Result<(), AgentError> {
    let s = spawn_agent_acp_session(client, args.cwd).await?;
    if let Err(e) = s.prompt(args.kpop_prompt, args.kpop_log).await {
        let _ = s.shutdown().await;
        return Err(AgentError(e));
    }
        maybe_tee_log(client, args.kpop_log);
    if let Some((learn_body, learn_log)) = args.learn {
        if let Err(e) = s.prompt(learn_body, learn_log).await {
            let _ = s.shutdown().await;
            return Err(AgentError(e));
        }
        maybe_tee_log(client, learn_log);
    }
    s.shutdown().await.map_err(AgentError)
}
