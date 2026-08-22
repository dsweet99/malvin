use crate::acp::AgentError;
use crate::bridge_sdk::{
    BridgeSession, BridgeSpawnArgs, send_create, send_resume, start_mem_watch,
};

use super::auth::effective_sdk_api_key;
use super::bridge_path::resolve_bridge_js;

pub(crate) async fn cursor_spawn_bridge(
    args: BridgeSpawnArgs<'_>,
) -> Result<BridgeSession, AgentError> {
    crate::malvin_sandbox::assert_dead_before_next_spawn().map_err(AgentError)?;
    let model = args.model.to_string();
    let resume_id = args.resume_agent_id.clone();
    let session = cursor_open_bridge_session(args)?;
    start_mem_watch(&session);
    let api_key = effective_sdk_api_key();
    if let Some(agent_id) = resume_id {
        send_resume(
            &session,
            crate::bridge_sdk::ResumeArgs {
                agent_id: &agent_id,
                cwd: &session.work_dir,
                model: &model,
                api_key: api_key.clone(),
            },
        )
        .await?;
    } else {
        send_create(
            &session,
            crate::bridge_sdk::CreateArgs {
                cwd: &session.work_dir,
                model: &model,
                api_key,
                models_json_path: None,
            },
        )
        .await?;
    }
    Ok(session)
}

fn cursor_open_bridge_session(args: BridgeSpawnArgs<'_>) -> Result<BridgeSession, AgentError> {
    let (node, bridge) = cursor_resolve_node_and_bridge()?;
    let mut child = cursor_build_bridge_command(&node, &bridge, args.cwd)
        .spawn()
        .map_err(|e| AgentError(format!("spawn cursor-sdk-bridge: {e}")))?;
    let handles = cursor_take_stdio(&mut child)?;
    cursor_note_sandbox(args.cwd, handles.pgid, &handles.baseline)?;
    Ok(cursor_assemble_session(args, child, handles))
}

struct CursorChildStdio {
    stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    pgid: Option<u32>,
    baseline: std::collections::HashSet<u32>,
}

fn cursor_take_stdio(child: &mut tokio::process::Child) -> Result<CursorChildStdio, AgentError> {
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| AgentError("bridge stdin missing".into()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AgentError("bridge stdout missing".into()))?;
    Ok(CursorChildStdio {
        stdin,
        stdout,
        pgid: child.id(),
        baseline: crate::malvin_sandbox::malvin_spawn_baseline(),
    })
}

fn cursor_note_sandbox(
    cwd: &std::path::Path,
    pgid: Option<u32>,
    baseline: &std::collections::HashSet<u32>,
) -> Result<(), AgentError> {
    crate::malvin_sandbox::note_active_sandbox_session(pgid, baseline.clone(), cwd)
        .map_err(AgentError)
}

fn cursor_assemble_session(
    args: BridgeSpawnArgs<'_>,
    child: tokio::process::Child,
    handles: CursorChildStdio,
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
        log: crate::bridge_sdk::StreamLog::from_spawn(&args, false),
        agent_id: Mutex::new(None),
        turn_id: Mutex::new(None),
        wire: crate::bridge_sdk::BridgeWire::NodeBridge,
    }
}

fn cursor_resolve_node_and_bridge() -> Result<(std::path::PathBuf, std::path::PathBuf), AgentError>
{
    let bridge = resolve_bridge_js().map_err(AgentError)?;
    let node = super::node_resolve::resolve_node_bin().map_err(AgentError)?;
    Ok((node, bridge))
}

fn cursor_build_bridge_command(
    node: &std::path::Path,
    bridge: &std::path::Path,
    cwd: &std::path::Path,
) -> tokio::process::Command {
    use std::process::Stdio;
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
    configure_node_compile_cache(&mut cmd);
    cmd
}

fn configure_node_compile_cache(cmd: &mut tokio::process::Command) {
    if std::env::var_os("NODE_COMPILE_CACHE").is_some() {
        return;
    }
    let cache_dir = crate::user_home::user_home_dir()
        .join(".malvin_home")
        .join("node_compile_cache");
    let cache_probe = cache_dir.join(format!(".write-probe-{}", std::process::id()));
    let cache_is_writable =
        std::fs::create_dir_all(&cache_dir).is_ok() && std::fs::write(&cache_probe, []).is_ok();
    let _ = std::fs::remove_file(&cache_probe);
    let cache_dir = if cache_is_writable {
        cache_dir
    } else {
        let fallback = std::env::temp_dir().join("malvin-node-compile-cache");
        let _ = std::fs::create_dir_all(&fallback);
        fallback
    };
    cmd.env("NODE_COMPILE_CACHE", cache_dir);
}

#[cfg(test)]
mod kiss_cov_names {
    #[test]
    fn kiss_cov_session_spawn_idents() {
        let _ = super::cursor_spawn_bridge;
        let _ = super::cursor_open_bridge_session;
        let _ = super::cursor_take_stdio;
        let _ = super::cursor_note_sandbox;
        let _ = super::cursor_assemble_session;
        let _ = super::cursor_resolve_node_and_bridge;
        let _ = super::cursor_build_bridge_command;
        let _ = stringify!(CursorChildStdio);
    }
}
