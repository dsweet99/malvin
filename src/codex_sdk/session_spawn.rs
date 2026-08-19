use crate::acp::AgentError;
use crate::bridge_sdk::{BridgeSession, BridgeSpawnArgs, BridgeWire, start_mem_watch};
use std::process::Stdio;

pub(crate) async fn codex_spawn_bridge(
    args: BridgeSpawnArgs<'_>,
) -> Result<BridgeSession, AgentError> {
    if !args.io.force {
        return Err(AgentError(
            "--no-force is not supported for codex: (malvin runs Codex tools headlessly; no interactive approval)"
                .into(),
        ));
    }
    crate::malvin_sandbox::assert_dead_before_next_spawn().map_err(AgentError)?;
    let bin = crate::codex_sdk::resolve_codex_bin().map_err(AgentError)?;
    let mut cmd = crate::malvin_sandbox::malvin_tokio_command(bin);
    cmd.arg("app-server")
        .current_dir(args.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .env("MALLOC_ARENA_MAX", "2");
    let mut child = cmd
        .spawn()
        .map_err(|e| AgentError(format!("spawn codex app-server: {e}")))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| AgentError("codex stdin missing".into()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AgentError("codex stdout missing".into()))?;
    let baseline = crate::malvin_sandbox::malvin_spawn_baseline();
    let pgid = child.id();
    crate::malvin_sandbox::note_active_sandbox_session(pgid, baseline.clone(), args.cwd)
        .map_err(AgentError)?;
    let session = BridgeSession {
        child: tokio::sync::Mutex::new(Some(child)),
        stdin: std::sync::Arc::new(tokio::sync::Mutex::new(stdin)),
        stdout: std::sync::Arc::new(tokio::sync::Mutex::new(tokio::io::BufReader::new(stdout))),
        process_group_id: pgid,
        spawn_pid_baseline: baseline,
        reader_dead: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        work_dir: args.cwd.to_path_buf(),
        io: args.io,
        last_response: std::sync::Arc::new(std::sync::Mutex::new(String::new())),
        timing: args.timing,
        run_dir: args.run_dir,
        started_at: std::time::Instant::now(),
        agent_id: std::sync::Mutex::new(None),
        stdout_coalesce: std::sync::Mutex::new(crate::acp::TraceChunkCoalescer::default()),
        tool_starts: std::sync::Mutex::new(std::collections::HashMap::new()),
        normalize_pi_usage: false,
        wire: BridgeWire::CodexRpc,
    };
    start_mem_watch(&session);
    codex_initialize(&session).await?;
    codex_start_thread(&session, args.model, args.cwd).await?;
    Ok(session)
}

pub(crate) async fn codex_initialize(session: &BridgeSession) -> Result<(), AgentError> {
    let response = request(
        session,
        "initialize",
        serde_json::json!({
            "clientInfo": {
                "name": "malvin",
                "title": "Malvin",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
    .await?;
    if response.get("error").is_some() {
        return Err(response_error("codex initialize", &response));
    }
    write(
        session,
        &serde_json::json!({"method":"initialized","params":{}}),
    )
    .await
}

pub(crate) async fn codex_start_thread(
    session: &BridgeSession,
    model: &str,
    cwd: &std::path::Path,
) -> Result<(), AgentError> {
    let response = request(
        session,
        "thread/start",
        serde_json::json!({
            "model": model,
            "cwd": cwd,
            "approvalPolicy": "never",
            "sandbox": "workspace-write"
        }),
    )
    .await?;
    if response.get("error").is_some() {
        return Err(response_error("codex thread/start", &response));
    }
    let id = response
        .pointer("/result/thread/id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError("codex thread/start response missing thread id".into()))?;
    *session
        .agent_id
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(id.to_owned());
    Ok(())
}

pub(crate) async fn request(
    session: &BridgeSession,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, AgentError> {
    let id = super::session_io::next_id();
    write(
        session,
        &serde_json::json!({"method": method, "id": id, "params": params}),
    )
    .await?;
    loop {
        let value = super::session_io::read_json(session).await?;
        if value.get("id").and_then(serde_json::Value::as_u64) == Some(id) {
            return Ok(value);
        }
    }
}

pub(crate) fn response_error(context: &str, response: &serde_json::Value) -> AgentError {
    AgentError(format!(
        "{context}: {}",
        response.get("error").unwrap_or(response)
    ))
}

async fn write(session: &BridgeSession, value: &serde_json::Value) -> Result<(), AgentError> {
    super::session_io::write_json(session, value).await
}
