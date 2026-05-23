pub(crate) const MALVIN_TEST_NO_REAL_AGENT_ENV: &str = "MALVIN_TEST_NO_REAL_AGENT";

pub(crate) fn test_no_real_agent_enabled() -> bool {
    std::env::var_os(MALVIN_TEST_NO_REAL_AGENT_ENV)
        .is_some_and(|v| !v.is_empty() && v != "0")
}

pub(crate) fn resolve_agent_bin() -> Option<PathBuf> {
    let explicit = std::env::var_os("MALVIN_AGENT_ACP_BIN")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from);
    if explicit.is_some() || test_no_real_agent_enabled() {
        explicit
    } else {
        crate::support_paths::agent_or_cursor_agent_bin()
    }
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
    if test_no_real_agent_enabled() {
        return false;
    }
    StdCommand::new(args[0])
        .args(&args[1..])
        .output()
        .is_ok_and(|o| o.status.success())
}

pub(crate) async fn spawn_agent_acp_session(client: &AgentClient, cwd: &Path) -> Result<AcpSession, AgentError> {
    let bin = resolve_agent_bin();
    if bin.is_none() && test_no_real_agent_enabled() {
        return Err(AgentError(
            "real Cursor agent is disabled for tests; set MALVIN_AGENT_ACP_BIN to an ACP mock"
                .to_string(),
        ));
    }
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

pub(crate) fn client_timing_elapsed_ms(client: &AgentClient) -> u64 {
    client.timing.as_ref().map_or(0, |t| {
        let d = t
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .elapsed_so_far();
        u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
    })
}


