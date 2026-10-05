use super::discover::resolve_codex_bin;
use super::session::CodexSession;
use crate::agent_process::AgentError;
use crate::backends::bridge_sdk::BridgeSpawnArgs;
use crate::agent_process::malvin_sandbox::SandboxSpawnTicket;
use std::process::Stdio;

pub(super) type CodexProcess = crate::backends::bridge_sdk::SpawnedStdio;

pub(super) fn spawn_codex_session(
    args: &BridgeSpawnArgs<'_>,
    service: Option<&str>,
    ticket: SandboxSpawnTicket,
) -> Result<CodexSession, AgentError> {
    let mut process = spawn_codex_process(args)?;
    process.baseline_pids = crate::agent_process::malvin_sandbox::malvin_spawn_baseline();
    crate::agent_process::malvin_sandbox::note_active_sandbox_session(
        ticket,
        process.pgid,
        process.baseline_pids.clone(),
        args.cwd,
    )
    .map_err(AgentError)?;
    Ok(build_codex_session(args, process, service))
}

pub(super) fn build_codex_session(
    args: &BridgeSpawnArgs<'_>,
    process: CodexProcess,
    service: Option<&str>,
) -> CodexSession {
    CodexSession {
        stdio: crate::backends::bridge_sdk::StdioChild::new(
            process,
            args.cwd.to_path_buf(),
            crate::backends::bridge_sdk::StreamLog::from_spawn(args),
        ),
        thread_id: std::sync::Mutex::new(None),
        turn_id: std::sync::Mutex::new(None),
        service: service.map(str::to_owned),
    }
}

pub(crate) const CODEX_OUTER_SANDBOX_ENV: &str = "MALVIN_CODEX_OUTER_SANDBOX";

pub(crate) fn codex_uses_outer_sandbox() -> bool {
    codex_uses_outer_sandbox_value(std::env::var(CODEX_OUTER_SANDBOX_ENV).ok().as_deref())
}

fn codex_uses_outer_sandbox_value(value: Option<&str>) -> bool {
    value == Some("1")
}

fn configure_codex_sandbox(cmd: &mut tokio::process::Command, outer_sandbox: bool) {
    if outer_sandbox {
        cmd.arg("--dangerously-bypass-approvals-and-sandbox");
    }
    cmd.arg("-c").arg("sandbox_mode=\"danger-full-access\"");
}

pub(super) fn configured_codex_command(
    bin: std::path::PathBuf,
    cwd: &std::path::Path,
) -> tokio::process::Command {
    let mut cmd = crate::agent_process::malvin_sandbox::malvin_tokio_command(bin);
    configure_codex_sandbox(&mut cmd, codex_uses_outer_sandbox());
    cmd.arg("app-server")
        .arg("--stdio")
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

pub(super) fn spawn_codex_process(args: &BridgeSpawnArgs<'_>) -> Result<CodexProcess, AgentError> {
    let bin = resolve_codex_bin().map_err(AgentError)?;
    let mut cmd = configured_codex_command(bin, args.cwd);
    let mut child = cmd
        .spawn()
        .map_err(|e| AgentError(format!("spawn codex app-server: {e}")))?;
    let (stdin, stdout) = crate::backends::bridge_sdk::take_stdio_forward_stderr(&mut child, "codex")?;
    Ok(crate::backends::bridge_sdk::SpawnedStdio::new(child, stdin, stdout))
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
        let _ = crate::backends::bridge_sdk::take_stdio_forward_stderr;
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
        assert_eq!(
            args,
            [
                "-c",
                "sandbox_mode=\"danger-full-access\"",
                "app-server",
                "--stdio",
            ]
        );
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

    #[test]
    fn configure_codex_sandbox_sets_mode_without_outer_bypass() {
        let mut cmd = tokio::process::Command::new("codex");
        configure_codex_sandbox(&mut cmd, false);
        let args: Vec<_> = cmd
            .as_std()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args, ["-c", "sandbox_mode=\"danger-full-access\"",]);
    }
}
