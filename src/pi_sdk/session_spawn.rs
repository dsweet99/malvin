
use crate::acp::AgentError;
use crate::bridge_sdk::{start_mem_watch, BridgeSession, BridgeSpawnArgs, BridgeWire};

use super::discover::{pi_version_ok, resolve_pi_bin};
use super::session_io::pi_send_new_session;

pub(crate) async fn pi_spawn_bridge(args: BridgeSpawnArgs<'_>) -> Result<BridgeSession, AgentError> {
    if !args.io.force {
        return Err(AgentError(
            "--no-force is not supported for pi: (malvin runs Pi tools headlessly; no interactive approval)"
                .into(),
        ));
    }
    crate::malvin_sandbox::assert_dead_before_next_spawn().map_err(AgentError)?;
    let (provider, model) = split_provider_model(args.model)?;
    let bin = resolve_pi_bin().map_err(AgentError)?;
    pi_version_ok(&bin).map_err(AgentError)?;
    let session = pi_open_bridge_session(args, &bin, provider, model)?;
    start_mem_watch(&session);
    pi_send_new_session(&session).await?;
    Ok(session)
}

pub(crate) fn split_provider_model(slug: &str) -> Result<(&str, &str), AgentError> {
    let Some((provider, model)) = slug.split_once('/') else {
        return Err(AgentError(format!(
            "pi model id must be `pi:<provider>/<model>` (got slug `{slug}`)"
        )));
    };
    if provider.is_empty() || model.is_empty() {
        return Err(AgentError(format!(
            "pi model id must be `pi:<provider>/<model>` (got slug `{slug}`)"
        )));
    }
    Ok((provider, model))
}

fn pi_open_bridge_session(
    args: BridgeSpawnArgs<'_>,
    bin: &std::path::Path,
    provider: &str,
    model: &str,
) -> Result<BridgeSession, AgentError> {
    let mut child = pi_build_command(bin, &args, provider, model)
        .spawn()
        .map_err(|e| AgentError(format!("spawn pi: {e}")))?;
    let handles = pi_take_stdio(&mut child)?;
    pi_note_sandbox(args.cwd, handles.pgid, &handles.baseline)?;
    Ok(pi_assemble_session(args, child, handles))
}

struct PiChildStdio {
    stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    pgid: Option<u32>,
    baseline: std::collections::HashSet<u32>,
}

fn pi_take_stdio(child: &mut tokio::process::Child) -> Result<PiChildStdio, AgentError> {
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| AgentError("pi stdin missing".into()))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AgentError("pi stdout missing".into()))?;
    Ok(PiChildStdio {
        stdin,
        stdout,
        pgid: child.id(),
        baseline: crate::malvin_sandbox::malvin_spawn_baseline(),
    })
}

fn pi_note_sandbox(
    cwd: &std::path::Path,
    pgid: Option<u32>,
    baseline: &std::collections::HashSet<u32>,
) -> Result<(), AgentError> {
    crate::malvin_sandbox::note_active_sandbox_session(pgid, baseline.clone(), cwd)
        .map_err(AgentError)
}

fn pi_assemble_session(
    args: BridgeSpawnArgs<'_>,
    child: tokio::process::Child,
    handles: PiChildStdio,
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
        normalize_pi_usage: true,
        wire: BridgeWire::PiRpc,
    }
}

fn pi_build_command(
    bin: &std::path::Path,
    args: &BridgeSpawnArgs<'_>,
    provider: &str,
    model: &str,
) -> tokio::process::Command {
    use std::process::Stdio;
    let mut cmd = crate::malvin_sandbox::malvin_tokio_command(bin);
    cmd.arg("--rpc")
        .arg("--provider")
        .arg(provider)
        .arg("--model")
        .arg(model);
    if let Some(level) = args.thinking {
        cmd.arg("--thinking").arg(level);
    }
    cmd.arg("--no-session")
        .arg("--no-extensions")
        .current_dir(args.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .env("MALLOC_ARENA_MAX", "2");
    cmd
}

#[cfg(test)]
mod thinking_arg_tests {
    use super::*;

    #[test]
    fn pi_command_forwards_thinking_separately_from_model() {
        let cwd = tempfile::tempdir().expect("tempdir");
        let args = BridgeSpawnArgs {
            cwd: cwd.path(),
            model: "openai/gpt-5",
            thinking: Some("high"),
            io: crate::agent_backend::test_support::test_io(),
            run_dir: None,
            timing: None,
            resume_agent_id: None,
            normalize_pi_usage: true,
        };
        let command = pi_build_command(
            std::path::Path::new("/usr/bin/pi"),
            &args,
            "openai",
            "gpt-5",
        );
        let child_args = command
            .as_std()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert_eq!(
            child_args,
            [
                "--rpc",
                "--provider",
                "openai",
                "--model",
                "gpt-5",
                "--thinking",
                "high",
                "--no-session",
                "--no-extensions",
            ]
        );
    }
}
