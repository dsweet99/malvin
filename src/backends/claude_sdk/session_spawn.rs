use crate::agent_process::AgentError;
use crate::backends::bridge_sdk::{
    BridgeSession, BridgeSpawnArgs, CreateArgs, SpawnedStdio, StdioChild, StreamLog, send_create,
    start_mem_watch,
};

use super::claude_sdk::bridge_path::resolve_bridge_js;

pub(crate) async fn claude_spawn_bridge(
    args: BridgeSpawnArgs<'_>,
) -> Result<BridgeSession, AgentError> {
    let ticket =
        crate::agent_process::malvin_sandbox::take_sandbox_spawn_ticket().map_err(AgentError)?;
    let session = open_session(&args, ticket)?;
    start_mem_watch(session.stdio.mem_watch_args());
    initialize_session(&session, &args).await?;
    Ok(session)
}

fn open_session(
    args: &BridgeSpawnArgs<'_>,
    ticket: crate::agent_process::malvin_sandbox::SandboxSpawnTicket,
) -> Result<BridgeSession, AgentError> {
    let mut child = spawn_bridge_process(args)?;
    let (stdin, stdout, stderr) = take_stdio(&mut child)?;
    crate::backends::cursor_sdk::bridge_stderr::start_filtered_forward(stderr);
    let pgid = child.id();
    let baseline = crate::agent_process::malvin_sandbox::malvin_spawn_baseline();
    register_sandbox(ticket, pgid, &baseline, args.cwd)?;
    Ok(BridgeSession {
        stdio: StdioChild::new(
            SpawnedStdio {
                child,
                stdin,
                stdout,
                pgid,
                baseline_pids: baseline,
            },
            args.cwd.to_path_buf(),
            StreamLog::from_spawn(args),
        ),
        agent_id: std::sync::Mutex::new(None),
    })
}

fn take_stdio(
    child: &mut tokio::process::Child,
) -> Result<
    (
        tokio::process::ChildStdin,
        tokio::process::ChildStdout,
        tokio::process::ChildStderr,
    ),
    AgentError,
> {
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| AgentError("Claude bridge stdin missing".into()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AgentError("Claude bridge stdout missing".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AgentError("Claude bridge stderr missing".into()))?;
    Ok((stdin, stdout, stderr))
}

fn register_sandbox(
    ticket: crate::agent_process::malvin_sandbox::SandboxSpawnTicket,
    pgid: Option<u32>,
    baseline: &std::collections::HashSet<u32>,
    cwd: &std::path::Path,
) -> Result<(), AgentError> {
    crate::agent_process::malvin_sandbox::note_active_sandbox_session(
        ticket,
        pgid,
        baseline.clone(),
        cwd,
    )
    .map_err(AgentError)
}

fn spawn_bridge_process(args: &BridgeSpawnArgs<'_>) -> Result<tokio::process::Child, AgentError> {
    let bridge = resolve_bridge_js().map_err(AgentError)?;
    let node = crate::backends::cursor_sdk::node_resolve::resolve_node_bin().map_err(AgentError)?;
    let mut command = crate::agent_process::malvin_sandbox::malvin_tokio_command(node);
    crate::backends::cursor_sdk::node_resolve::apply_quiet_node_cli(&mut command);
    command
        .arg(bridge)
        .current_dir(args.cwd)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    for param in &args.model.params {
        match param.id.as_str() {
            "effort" => command.env("MALVIN_CLAUDE_EFFORT", &param.value),
            "fast" => command.env("MALVIN_CLAUDE_FAST", &param.value),
            _ => &mut command,
        };
    }
    command
        .spawn()
        .map_err(|error| AgentError(format!("spawn claude-sdk-bridge: {error}")))
}

async fn initialize_session(
    session: &BridgeSession,
    args: &BridgeSpawnArgs<'_>,
) -> Result<(), AgentError> {
    send_create(
        session,
        CreateArgs {
            cwd: args.cwd,
            model: &args.wire_model(),
            api_key: None,
            models_json_path: None,
        },
    )
    .await
}
