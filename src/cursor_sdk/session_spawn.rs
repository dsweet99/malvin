//! Bridge process spawn helpers.

use crate::acp::AgentError;

use super::bridge_path::resolve_bridge_js;
use super::session::{BridgeSession, BridgeSpawnArgs};

pub(super) async fn spawn_bridge(args: BridgeSpawnArgs<'_>) -> Result<BridgeSession, AgentError> {
    crate::malvin_sandbox::assert_dead_before_next_spawn().map_err(AgentError)?;
    let model = args.model.to_string();
    let session = open_bridge_session(args)?;
    super::session_io::start_mem_watch(&session);
    super::session_io::send_create(&session, &session.work_dir, &model).await?;
    Ok(session)
}

fn open_bridge_session(args: BridgeSpawnArgs<'_>) -> Result<BridgeSession, AgentError> {
    let (node, bridge) = resolve_node_and_bridge()?;
    let mut child = build_bridge_command(&node, &bridge, args.cwd)
        .spawn()
        .map_err(|e| AgentError(format!("spawn cursor-sdk-bridge: {e}")))?;
    let handles = take_stdio(&mut child)?;
    note_sandbox(args.cwd, handles.pgid, &handles.baseline)?;
    Ok(assemble_session(args, child, handles))
}

struct ChildStdio {
    stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    pgid: Option<u32>,
    baseline: std::collections::HashSet<u32>,
}

fn take_stdio(child: &mut tokio::process::Child) -> Result<ChildStdio, AgentError> {
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| AgentError("bridge stdin missing".into()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AgentError("bridge stdout missing".into()))?;
    Ok(ChildStdio {
        stdin,
        stdout,
        pgid: child.id(),
        baseline: crate::malvin_sandbox::malvin_spawn_baseline(),
    })
}

fn note_sandbox(
    cwd: &std::path::Path,
    pgid: Option<u32>,
    baseline: &std::collections::HashSet<u32>,
) -> Result<(), AgentError> {
    crate::malvin_sandbox::note_active_sandbox_session(pgid, baseline.clone(), cwd)
        .map_err(AgentError)
}

fn assemble_session(
    args: BridgeSpawnArgs<'_>,
    child: tokio::process::Child,
    handles: ChildStdio,
) -> BridgeSession {
    use std::sync::atomic::AtomicBool;
    use std::sync::{Arc, Mutex};
    use tokio::io::BufReader;
    use tokio::sync::Mutex as AsyncMutex;
    BridgeSession {
        child: AsyncMutex::new(Some(child)),
        stdin: Arc::new(AsyncMutex::new(handles.stdin)),
        stdout: Arc::new(AsyncMutex::new(BufReader::new(handles.stdout))),
        process_group_id: handles.pgid,
        spawn_pid_baseline: handles.baseline,
        reader_dead: Arc::new(AtomicBool::new(false)),
        work_dir: args.cwd.to_path_buf(),
        io: args.io,
        last_response: Arc::new(Mutex::new(String::new())),
        timing: args.timing,
        run_dir: args.run_dir,
        started_at: std::time::Instant::now(),
        stdout_coalesce: Mutex::new(crate::acp::TraceChunkCoalescer::default()),
        tool_starts: Mutex::new(std::collections::HashMap::new()),
    }
}

fn resolve_node_and_bridge() -> Result<(std::path::PathBuf, std::path::PathBuf), AgentError> {
    let bridge = resolve_bridge_js().map_err(AgentError)?;
    let node = super::node_resolve::resolve_node_bin().map_err(AgentError)?;
    Ok((node, bridge))
}

fn build_bridge_command(
    node: &std::path::Path,
    bridge: &std::path::Path,
    cwd: &std::path::Path,
) -> tokio::process::Command {
    use std::process::Stdio;
    use super::auth::effective_sdk_api_key;
    let mut cmd = crate::malvin_sandbox::malvin_tokio_command(node);
    super::node_resolve::apply_quiet_node_cli(&mut cmd);
    cmd.arg(bridge)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .env("MALLOC_ARENA_MAX", "2");
    if let Some(k) = effective_sdk_api_key() {
        cmd.env("CURSOR_API_KEY", k);
    }
    // ~100–140 ms off repeated @cursor/sdk imports after the cache fills (ideas.md #6).
    if std::env::var_os("NODE_COMPILE_CACHE").is_none() {
        let cache_dir = crate::user_home::user_home_dir()
            .join(".malvin_home")
            .join("node_compile_cache");
        let _ = std::fs::create_dir_all(&cache_dir);
        cmd.env("NODE_COMPILE_CACHE", cache_dir);
    }
    cmd
}
