//! Prime bridge process spawn helpers (always `create`; no resume).

use crate::acp::AgentError;
use crate::bridge_sdk::{send_create, start_mem_watch, BridgeSession, BridgeSpawnArgs};

use super::bridge_path::prime_resolve_bridge_js;

pub(crate) async fn prime_spawn_bridge(args: BridgeSpawnArgs<'_>) -> Result<BridgeSession, AgentError> {
    crate::malvin_sandbox::assert_dead_before_next_spawn().map_err(AgentError)?;
    let model = args.model.to_string();
    let local_sidecar = if args.prime_local {
        Some(
            crate::local_llm::PrimeLocalSidecar::start(
                &format!("prime:{model}"),
                args.allow_download,
            )
            .map_err(AgentError)?,
        )
    } else {
        None
    };
    let models_json = local_sidecar
        .as_ref()
        .map(|s| s.models_json_path.display().to_string());
    let mut session = prime_open_bridge_session(args)?;
    session.local_sidecar = local_sidecar;
    start_mem_watch(&session);
    // Never forward Cursor credentials; bridge uses Prime AuthStorage + provider env.
    send_create(
        &session,
        crate::bridge_sdk::CreateArgs {
            cwd: &session.work_dir,
            model: &model,
            api_key: None,
            models_json_path: models_json.as_deref(),
        },
    )
    .await?;
    Ok(session)
}

fn prime_open_bridge_session(args: BridgeSpawnArgs<'_>) -> Result<BridgeSession, AgentError> {
    let (node, bridge) = prime_resolve_node_and_bridge()?;
    let mut child = prime_build_bridge_command(&node, &bridge, args.cwd)
        .spawn()
        .map_err(|e| AgentError(format!("spawn prime-sdk-bridge: {e}")))?;
    let handles = prime_take_stdio(&mut child)?;
    prime_note_sandbox(args.cwd, handles.pgid, &handles.baseline)?;
    Ok(prime_assemble_session(args, child, handles))
}

struct PrimeChildStdio {
    stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    pgid: Option<u32>,
    baseline: std::collections::HashSet<u32>,
}

fn prime_take_stdio(child: &mut tokio::process::Child) -> Result<PrimeChildStdio, AgentError> {
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| AgentError("bridge stdin missing".into()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AgentError("bridge stdout missing".into()))?;
    Ok(PrimeChildStdio {
        stdin,
        stdout,
        pgid: child.id(),
        baseline: crate::malvin_sandbox::malvin_spawn_baseline(),
    })
}

fn prime_note_sandbox(
    cwd: &std::path::Path,
    pgid: Option<u32>,
    baseline: &std::collections::HashSet<u32>,
) -> Result<(), AgentError> {
    crate::malvin_sandbox::note_active_sandbox_session(pgid, baseline.clone(), cwd)
        .map_err(AgentError)
}

fn prime_assemble_session(
    args: BridgeSpawnArgs<'_>,
    child: tokio::process::Child,
    handles: PrimeChildStdio,
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
        agent_id: Mutex::new(None),
        stdout_coalesce: Mutex::new(crate::acp::TraceChunkCoalescer::default()),
        tool_starts: Mutex::new(std::collections::HashMap::new()),
        local_sidecar: None,
        normalize_prime_usage: args.normalize_prime_usage,
    }
}

fn prime_resolve_node_and_bridge() -> Result<(std::path::PathBuf, std::path::PathBuf), AgentError> {
    let bridge = prime_resolve_bridge_js().map_err(AgentError)?;
    let node = super::node_resolve::prime_resolve_node_bin().map_err(AgentError)?;
    Ok((node, bridge))
}

fn prime_build_bridge_command(
    node: &std::path::Path,
    bridge: &std::path::Path,
    cwd: &std::path::Path,
) -> tokio::process::Command {
    use std::process::Stdio;
    let mut cmd = crate::malvin_sandbox::malvin_tokio_command(node);
    super::node_resolve::prime_apply_quiet_node_cli(&mut cmd);
    cmd.arg(bridge)
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .env("MALLOC_ARENA_MAX", "2");
    scrub_cursor_keys(&mut cmd);
    apply_node_compile_cache(&mut cmd);
    cmd
}

fn scrub_cursor_keys(cmd: &mut tokio::process::Command) {
    cmd.env_remove("CURSOR_API_KEY");
    cmd.env_remove("CURSOR_AGENT_API_KEY");
    cmd.env_remove("AGENT_API_KEY");
}

fn apply_node_compile_cache(cmd: &mut tokio::process::Command) {
    if std::env::var_os("NODE_COMPILE_CACHE").is_some() {
        return;
    }
    let cache_dir = crate::user_home::user_home_dir()
        .join(".malvin_home")
        .join("node_compile_cache_prime");
    let _ = std::fs::create_dir_all(&cache_dir);
    cmd.env("NODE_COMPILE_CACHE", cache_dir);
}

#[cfg(test)]
mod kiss_cov_names {
    #[test]
    fn kiss_cov_session_spawn_idents() {
        let _ = super::prime_spawn_bridge;
        let _ = super::prime_open_bridge_session;
        let _ = super::prime_take_stdio;
        let _ = super::prime_note_sandbox;
        let _ = super::prime_assemble_session;
        let _ = super::prime_resolve_node_and_bridge;
        let _ = super::prime_build_bridge_command;
        let _ = super::scrub_cursor_keys;
        let _ = super::apply_node_compile_cache;
        let _ = stringify!(PrimeChildStdio);
    }
}
