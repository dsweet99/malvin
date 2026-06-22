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
    crate::malvin_sandbox::malvin_std_command(args[0])
        .args(&args[1..])
        .output()
        .is_ok_and(|o| o.status.success())
}

/// True when [`AgentClient::ensure_authenticated`] would succeed without an ACP `authenticate` RPC.
pub(crate) fn cursor_cli_auth_established() -> bool {
    if has_api_key() {
        return true;
    }
    auth_probe(&["agent", "auth", "status"])
        || auth_probe(&["cursor-agent", "auth", "status"])
        || auth_probe(&["agent", "whoami"])
}

pub(crate) fn resolve_acp_session_cwd(cwd: &Path) -> Result<PathBuf, AgentError> {
    let base = if cwd.as_os_str().is_empty() || cwd == Path::new(".") {
        std::env::current_dir().map_err(|e| AgentError(e.to_string()))?
    } else if cwd.is_absolute() {
        cwd.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| AgentError(e.to_string()))?
            .join(cwd)
    };
    Ok(base.canonicalize().unwrap_or(base))
}

pub(crate) async fn spawn_agent_acp_session(client: &AgentClient, cwd: &Path) -> Result<AcpSession, AgentError> {
    let bin = resolve_agent_bin();
    if bin.is_none() && test_no_real_agent_enabled() {
        return Err(AgentError(
            "real Cursor agent is disabled for tests; set MALVIN_AGENT_ACP_BIN to an ACP mock"
                .to_string(),
        ));
    }
    let cwd = resolve_acp_session_cwd(cwd)?;
    let rpc_secs = crate::support_paths::acp_rpc_timeout_secs_from_env();
    let model = client.model.trim();
    let model_opt = (!model.is_empty()).then_some(model);
    let mut session = AcpSession::spawn(AcpSpawnArgs {
        cwd: &cwd,
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
    .map_err(AgentError)?;
    crate::acp::acp_session_set_run_timing(&mut session, client.timing.clone());
    Ok(session)
}

#[cfg(test)]
mod ops_body_spawn_kiss {
    use super::*;

    #[test]
    fn smoke_ops_body_spawn_symbols() {
        let _ = test_no_real_agent_enabled();
        let _ = resolve_agent_bin();
        let _ = has_api_key();
        let _ = auth_probe(&["/bin/true"]);
        let _ = cursor_cli_auth_established();
    }

    #[test]
    fn resolve_acp_session_cwd_expands_dot() {
        let cwd = std::env::current_dir().expect("cwd");
        let resolved = resolve_acp_session_cwd(Path::new(".")).expect("resolve");
        assert!(resolved.is_absolute());
        assert_eq!(resolved, cwd.canonicalize().unwrap_or(cwd));
    }
}
