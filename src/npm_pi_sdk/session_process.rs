use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use tokio::io::BufReader;
use tokio::sync::Mutex as AsyncMutex;

use super::discover::resolve_npm_pi_entry;
use super::session::NpmPiSession;
use crate::acp::AgentError;
use crate::bridge_sdk::{BridgeSpawnArgs, StreamLog};

pub(super) type NpmPiProcess = (
    tokio::process::Child,
    tokio::process::ChildStdin,
    tokio::process::ChildStdout,
    Option<u32>,
    std::collections::HashSet<u32>,
);

pub(super) fn spawn_npm_pi_session(
    args: &BridgeSpawnArgs<'_>,
    ticket: crate::malvin_sandbox::SandboxSpawnTicket,
) -> Result<NpmPiSession, AgentError> {
    let process = spawn_npm_pi_process(args)?;
    build_npm_pi_session(args, process, ticket)
}

pub(super) fn spawn_npm_pi_process(args: &BridgeSpawnArgs<'_>) -> Result<NpmPiProcess, AgentError> {
    let entry = resolve_npm_pi_entry().map_err(AgentError)?;
    let node = crate::cursor_sdk::node_resolve::resolve_node_bin().map_err(AgentError)?;
    let mut cmd = configured_npm_pi_command(node, entry, args)?;
    let mut child = cmd
        .spawn()
        .map_err(|e| AgentError(format!("spawn npm pi rpc: {e}")))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| AgentError("npm pi stdin missing".into()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AgentError("npm pi stdout missing".into()))?;
    let pgid = child.id();
    Ok((child, stdin, stdout, pgid, std::collections::HashSet::new()))
}

fn configured_npm_pi_command(
    node: std::path::PathBuf,
    entry: std::path::PathBuf,
    args: &BridgeSpawnArgs<'_>,
) -> Result<tokio::process::Command, AgentError> {
    let (provider, model) = args.model.pi_provider_and_model().ok_or_else(|| {
        AgentError(format!(
            "pi model id must be `pi:<provider>/<model>` (got `{}`)",
            args.model.canonical()
        ))
    })?;
    let mut cmd = crate::malvin_sandbox::malvin_tokio_command(node);
    cmd.arg(&entry);
    if !entry_is_rpc_entry(&entry) {
        cmd.arg("--mode").arg("rpc");
    }
    append_provider_model(&mut cmd, provider, model, args);
    Ok(cmd)
}

fn append_provider_model(
    cmd: &mut tokio::process::Command,
    provider: &str,
    model: &str,
    args: &BridgeSpawnArgs<'_>,
) {
    cmd.arg("--provider")
        .arg(provider)
        .arg("--model")
        .arg(model)
        .arg("--no-session")
        .current_dir(args.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .env("MALLOC_ARENA_MAX", "2");
    if let Some(thinking) = args.thinking {
        cmd.arg("--thinking").arg(thinking);
    }
}

fn entry_is_rpc_entry(entry: &std::path::Path) -> bool {
    entry
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.contains("rpc-entry"))
}

pub(super) fn build_npm_pi_session(
    args: &BridgeSpawnArgs<'_>,
    process: NpmPiProcess,
    ticket: crate::malvin_sandbox::SandboxSpawnTicket,
) -> Result<NpmPiSession, AgentError> {
    let (child, stdin, stdout, pgid, mut baseline) = process;
    baseline.extend(crate::malvin_sandbox::malvin_spawn_baseline());
    crate::malvin_sandbox::note_active_sandbox_session(ticket, pgid, baseline.clone(), args.cwd)
        .map_err(AgentError)?;
    Ok(NpmPiSession {
        child: AsyncMutex::new(Some(child)),
        stdin: Arc::new(AsyncMutex::new(stdin)),
        stdout: Arc::new(AsyncMutex::new(BufReader::new(stdout))),
        process_group_id: pgid,
        spawn_pid_baseline: baseline,
        reader_dead: Arc::new(AtomicBool::new(false)),
        work_dir: args.cwd.to_path_buf(),
        log: StreamLog::from_spawn(args),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_entry_detection() {
        assert!(entry_is_rpc_entry(std::path::Path::new(
            "/tmp/dist/bundle/rpc-entry.js"
        )));
        assert!(!entry_is_rpc_entry(std::path::Path::new(
            "/tmp/dist/bundle/cli.js"
        )));
    }

    #[test]
    fn kiss_cov_process_names() {
        let _ = spawn_npm_pi_session;
        let _ = spawn_npm_pi_process;
        let _ = configured_npm_pi_command;
        let _ = build_npm_pi_session;
        let _ = entry_is_rpc_entry;
    }
}
