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

pub(super) fn spawn_codex_session(
    args: &BridgeSpawnArgs<'_>,
    service: Option<&str>,
) -> Result<BridgeSession, AgentError> {
    let mut process = spawn_codex_process(args)?;
    let baseline = crate::malvin_sandbox::malvin_spawn_baseline();
    process.4 = baseline;
    crate::malvin_sandbox::note_active_sandbox_session(process.3, process.4.clone(), args.cwd)
        .map_err(AgentError)?;
    Ok(build_codex_session(args, process, service))
}

pub(super) fn build_codex_session(
    args: &BridgeSpawnArgs<'_>,
    process: CodexProcess,
    service: Option<&str>,
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
        log: {
            let mut log = crate::bridge_sdk::StreamLog::from_spawn(args);
            log.last_response = io.3;
            log
        },
        agent_id: std::sync::Mutex::new(None),
        turn_id: std::sync::Mutex::new(None),
        service: service.map(str::to_owned),
        wire: BridgeWire::CodexRpc,
    }
}

type CodexSessionIo = (
    std::sync::Arc<tokio::sync::Mutex<tokio::process::ChildStdin>>,
    std::sync::Arc<tokio::sync::Mutex<tokio::io::BufReader<tokio::process::ChildStdout>>>,
    std::sync::Arc<std::sync::atomic::AtomicBool>,
    std::sync::Arc<std::sync::Mutex<String>>,
);

pub(crate) const CODEX_OUTER_SANDBOX_ENV: &str = "MALVIN_CODEX_OUTER_SANDBOX";

pub(crate) fn codex_uses_outer_sandbox() -> bool {
    codex_uses_outer_sandbox_value(std::env::var(CODEX_OUTER_SANDBOX_ENV).ok().as_deref())
}

fn codex_uses_outer_sandbox_value(value: Option<&str>) -> bool {
    value == Some("1")
}

fn configure_codex_sandbox(cmd: &mut tokio::process::Command, outer_sandbox: bool) {
    if outer_sandbox {
        cmd.arg("--dangerously-bypass-approvals-and-sandbox")
            .arg("-c")
            .arg("sandbox_mode=\"danger-full-access\"");
    }
}

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
    // Fast tasks already run in Docker, where Codex's nested bubblewrap
    // sandbox cannot create a user namespace. Ordinary Codex runs retain
    // Codex's workspace-write sandbox.
    configure_codex_sandbox(&mut cmd, codex_uses_outer_sandbox());
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
    use super::{
        CodexProcess, codex_uses_outer_sandbox_value, configure_codex_sandbox,
        configured_codex_command,
    };
    #[test]
    fn kiss_cov_codex_process_type() {
        let _: Option<CodexProcess> = None;
    }

    #[test]
    fn configured_codex_command_uses_default_sandbox() {
        let cmd = configured_codex_command(
            std::path::PathBuf::from("codex"),
            std::path::Path::new("/work"),
        );
        let args: Vec<_> = cmd
            .as_std()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, ["app-server", "--stdio",]);
    }

    #[test]
    fn codex_outer_sandbox_is_opt_in() {
        assert!(codex_uses_outer_sandbox_value(Some("1")));
        assert!(!codex_uses_outer_sandbox_value(Some("true")));
        assert!(!codex_uses_outer_sandbox_value(None));
    }

    #[test]
    fn configured_codex_command_uses_outer_sandbox_when_requested() {
        let mut cmd = tokio::process::Command::new("codex");
        configure_codex_sandbox(&mut cmd, true);
        let args: Vec<_> = cmd
            .as_std()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            [
                "--dangerously-bypass-approvals-and-sandbox",
                "-c",
                "sandbox_mode=\"danger-full-access\"",
            ]
        );
    }
}
