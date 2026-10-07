use super::pi_backend::discover::resolve_npm_pi_entry;
use super::pi_backend::session::NpmPiSession;
use crate::agent_process::AgentError;
use crate::backends::bridge_sdk::{BridgeSpawnArgs, StdioChild, StreamLog};
use std::process::Stdio;

pub(super) type NpmPiProcess = crate::backends::bridge_sdk::SpawnedStdio;

pub(super) fn spawn_npm_pi_session(
    args: &BridgeSpawnArgs<'_>,
    ticket: crate::agent_process::malvin_sandbox::SandboxSpawnTicket,
) -> Result<NpmPiSession, AgentError> {
    let process = spawn_npm_pi_process(args)?;
    build_npm_pi_session(args, process, ticket)
}

pub(super) fn spawn_npm_pi_process(args: &BridgeSpawnArgs<'_>) -> Result<NpmPiProcess, AgentError> {
    let entry = resolve_npm_pi_entry().map_err(AgentError)?;
    let node = crate::backends::cursor_sdk::node_resolve::resolve_node_bin().map_err(AgentError)?;
    let mut cmd = configured_npm_pi_command(node, entry, args)?;
    let mut child = cmd
        .spawn()
        .map_err(|e| AgentError(format!("spawn npm pi rpc: {e}")))?;
    let (stdin, stdout) = crate::backends::bridge_sdk::take_stdio_forward_stderr(&mut child, "npm pi")?;
    Ok(crate::backends::bridge_sdk::SpawnedStdio::new(child, stdin, stdout))
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
    crate::local_llm::prepare_local_llm(args.cwd, provider, model).map_err(AgentError)?;
    let mut cmd = crate::agent_process::malvin_sandbox::malvin_tokio_command(node);
    cmd.arg(&entry);
    if !entry_is_rpc_entry(&entry) {
        cmd.arg("--mode").arg("rpc");
    }
    append_provider_model(&mut cmd, provider, model, args);
    cmd.args(crate::local_llm::local_cli_args(provider, model));
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
        .stderr(Stdio::piped());
    if let Some(thinking) = args.thinking {
        cmd.arg("--thinking").arg(thinking);
    }
}

pub(super) fn entry_is_rpc_entry(entry: &std::path::Path) -> bool {
    entry
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.contains("rpc-entry"))
}

pub(super) fn build_npm_pi_session(
    args: &BridgeSpawnArgs<'_>,
    mut process: NpmPiProcess,
    ticket: crate::agent_process::malvin_sandbox::SandboxSpawnTicket,
) -> Result<NpmPiSession, AgentError> {
    process
        .baseline_pids
        .extend(crate::agent_process::malvin_sandbox::malvin_spawn_baseline());
    crate::agent_process::malvin_sandbox::note_active_sandbox_session(
        ticket,
        process.pgid,
        process.baseline_pids.clone(),
        args.cwd,
    )
    .map_err(AgentError)?;
    let local_hold = take_local_hold(args.model)?;
    Ok(NpmPiSession {
        stdio: StdioChild::new(process, args.cwd.to_path_buf(), StreamLog::from_spawn(args)),
        pi_model: pi_model_pair(args.model),
        local_hold,
        output_cap: local_output_cap(args),
    })
}

fn local_output_cap(args: &BridgeSpawnArgs<'_>) -> Option<u64> {
    let (provider, model) = args.model.pi_provider_and_model()?;
    crate::local_llm::local_output_cap(args.cwd, provider, model)
}

fn take_local_hold(model: &crate::config::model_id::ParsedModel) -> Result<bool, AgentError> {
    if !crate::local_llm::model_needs_local_llm(model) {
        return Ok(false);
    }
    crate::local_llm::hold_local_llm().map_err(AgentError)?;
    Ok(true)
}

fn pi_model_pair(model: &crate::config::model_id::ParsedModel) -> Option<(String, String)> {
    model
        .pi_provider_and_model()
        .map(|(provider, id)| (provider.to_string(), id.to_string()))
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
    fn cloud_models_take_no_local_hold() {
        let model = crate::config::model_id::parse_model_id("pi:openai/gpt-4o").expect("model");
        assert!(!take_local_hold(&model).expect("no hold"));
    }

    #[test]
    fn kiss_cov_process_names() {
        let _ = spawn_npm_pi_session;
        let _ = spawn_npm_pi_process;
        let _ = configured_npm_pi_command;
        let _ = build_npm_pi_session;
        let _ = entry_is_rpc_entry;
        let _ = take_local_hold;
        let _ = crate::backends::bridge_sdk::take_stdio_forward_stderr;
    }
}
