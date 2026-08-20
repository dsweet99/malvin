use super::discover::resolve_codex_bin;
use crate::acp::AgentError;
use crate::bridge_sdk::{BridgeSession, BridgeSpawnArgs, BridgeWire};
use std::process::Stdio;

pub(super) type CodexProcess = (
    tokio::process::Child,
    tokio::process::ChildStdin,
    tokio::process::ChildStdout,
    Option<u32>,
    std::collections::HashSet<u32>,
);

pub(super) fn spawn_codex_session(args: &BridgeSpawnArgs<'_>) -> Result<BridgeSession, AgentError> {
    let mut process = spawn_codex_process(args)?;
    let baseline = crate::malvin_sandbox::malvin_spawn_baseline();
    process.4 = baseline;
    crate::malvin_sandbox::note_active_sandbox_session(process.3, process.4.clone(), args.cwd)
        .map_err(AgentError)?;
    Ok(build_codex_session(args, process))
}

pub(super) fn build_codex_session(
    args: &BridgeSpawnArgs<'_>,
    process: CodexProcess,
) -> BridgeSession {
    let (child, stdin, stdout, pgid, baseline) = process;
    let io = build_codex_session_io(stdin, stdout);
    BridgeSession {
        child: tokio::sync::Mutex::new(Some(child)),
        stdin: io.0,
        stdout: io.1,
        process_group_id: pgid,
        spawn_pid_baseline: baseline,
        reader_dead: io.2,
        work_dir: args.cwd.to_path_buf(),
        io: args.io,
        last_response: io.3,
        timing: args.timing.clone(),
        run_dir: args.run_dir.clone(),
        started_at: std::time::Instant::now(),
        agent_id: std::sync::Mutex::new(None),
        turn_id: std::sync::Mutex::new(None),
        stdout_coalesce: std::sync::Mutex::new(crate::acp::TraceChunkCoalescer::default()),
        tool_starts: std::sync::Mutex::new(std::collections::HashMap::default()),
        normalize_pi_usage: false,
        wire: BridgeWire::CodexRpc,
    }
}

type CodexSessionIo = (
    std::sync::Arc<tokio::sync::Mutex<tokio::process::ChildStdin>>,
    std::sync::Arc<tokio::sync::Mutex<tokio::io::BufReader<tokio::process::ChildStdout>>>,
    std::sync::Arc<std::sync::atomic::AtomicBool>,
    std::sync::Arc<std::sync::Mutex<String>>,
);

pub(super) fn build_codex_session_io(
    stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
) -> CodexSessionIo {
    (
        std::sync::Arc::new(tokio::sync::Mutex::new(stdin)),
        std::sync::Arc::new(tokio::sync::Mutex::new(tokio::io::BufReader::new(stdout))),
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        std::sync::Arc::new(std::sync::Mutex::new(String::new())),
    )
}

pub(super) fn configured_codex_command(
    bin: std::path::PathBuf,
    cwd: &std::path::Path,
) -> tokio::process::Command {
    let mut cmd = crate::malvin_sandbox::malvin_tokio_command(bin);
    cmd.arg("app-server")
        .arg("--stdio")
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .env("MALLOC_ARENA_MAX", "2");
    cmd
}

pub(super) fn spawn_codex_process(args: &BridgeSpawnArgs<'_>) -> Result<CodexProcess, AgentError> {
    let bin = resolve_codex_bin().map_err(AgentError)?;
    let mut cmd = configured_codex_command(bin, args.cwd);
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
    let pgid = child.id();
    Ok((child, stdin, stdout, pgid, std::collections::HashSet::new()))
}

#[cfg(test)]
mod tests {
    use super::CodexProcess;
    #[test]
    fn kiss_cov_codex_process_type() {
        let _: Option<CodexProcess> = None;
    }
}
