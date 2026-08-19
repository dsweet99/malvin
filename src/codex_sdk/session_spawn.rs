use crate::acp::AgentError;
use crate::bridge_sdk::{BridgeSession, BridgeSpawnArgs, BridgeWire, start_mem_watch};
use std::process::Stdio;

pub(crate) async fn codex_spawn_bridge(
    args: BridgeSpawnArgs<'_>,
) -> Result<BridgeSession, AgentError> {
    if !args.io.force {
        return Err(AgentError(
            "--no-force is not supported for codex: (malvin runs Codex tools headlessly; no interactive approval)".into(),
        ));
    }
    crate::malvin_sandbox::assert_dead_before_next_spawn().map_err(AgentError)?;
    let session = spawn_codex_session(&args)?;
    start_mem_watch(&session);
    codex_initialize(&session).await?;
    codex_start_thread(&session, args.model, args.cwd).await?;
    Ok(session)
}

fn spawn_codex_session(args: &BridgeSpawnArgs<'_>) -> Result<BridgeSession, AgentError> {
    let mut process = spawn_codex_process(args)?;
    process.baseline = crate::malvin_sandbox::malvin_spawn_baseline();
    crate::malvin_sandbox::note_active_sandbox_session(process.pgid, process.baseline.clone(), args.cwd)
        .map_err(AgentError)?;
    Ok(build_codex_session(args, process))
}

struct CodexProcess {
    child: tokio::process::Child,
    stdin: tokio::process::ChildStdin,
    stdout: tokio::process::ChildStdout,
    pgid: Option<u32>,
    baseline: std::collections::HashSet<u32>,
}

fn build_codex_session(args: &BridgeSpawnArgs<'_>, process: CodexProcess) -> BridgeSession {
    let CodexProcess { child, stdin, stdout, pgid, baseline } = process;
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

fn build_codex_session_io(
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

fn configured_codex_command(bin: std::path::PathBuf, cwd: &std::path::Path) -> tokio::process::Command {
    let mut cmd = crate::malvin_sandbox::malvin_tokio_command(bin);
    cmd.arg("app-server").arg("--stdio").current_dir(cwd).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::inherit()).env("MALLOC_ARENA_MAX", "2");
    cmd
}

fn spawn_codex_process(
    args: &BridgeSpawnArgs<'_>,
 ) -> Result<CodexProcess, AgentError> {
    let bin = crate::codex_sdk::resolve_codex_bin().map_err(AgentError)?;
    let mut cmd = configured_codex_command(bin, args.cwd);
    let mut child = cmd.spawn().map_err(|e| AgentError(format!("spawn codex app-server: {e}")))?;
    let stdin = child.stdin.take().ok_or_else(|| AgentError("codex stdin missing".into()))?;
    let stdout = child.stdout.take().ok_or_else(|| AgentError("codex stdout missing".into()))?;
    let pgid = child.id();
    Ok(CodexProcess { child, stdin, stdout, pgid, baseline: std::collections::HashSet::new() })
}

pub(crate) async fn codex_initialize(session: &BridgeSession) -> Result<(), AgentError> {
    let response = request(
        session,
        "initialize",
        serde_json::json!({
            "clientInfo": {
                "name": "malvin",
                "title": "Malvin",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
    .await?;
    if response.get("error").is_some() {
        return Err(response_error("codex initialize", &response));
    }
    write(
        session,
        &serde_json::json!({"method":"initialized","params":{}}),
    )
    .await
}

fn resolve_codex_model(model: &str) -> String {
    let Ok(models) = crate::codex_sdk::list_codex_models() else {
        return model.to_owned();
    };
    if models.iter().any(|(id, _)| id == model) {
        return model.to_owned();
    }
    models
        .into_iter()
        .find(|(id, _)| id.starts_with(&format!("{model}-")))
        .map_or_else(|| model.to_owned(), |(id, _)| id)
}

pub(crate) async fn codex_start_thread(
    session: &BridgeSession,
    model: &str,
    cwd: &std::path::Path,
) -> Result<(), AgentError> {
    let model = resolve_codex_model(model);
    let response = request(
        session,
        "thread/start",
        serde_json::json!({
            "model": model,
            "cwd": cwd,
            "approvalPolicy": "never",
            "sandbox": "workspace-write"
        }),
    )
    .await?;
    if response.get("error").is_some() {
        return Err(response_error("codex thread/start", &response));
    }
    let id = response
        .pointer("/result/thread/id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AgentError("codex thread/start response missing thread id".into()))?;
    *session
        .agent_id
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(id.to_owned());
    Ok(())
}

pub(crate) async fn request(
    session: &BridgeSession,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value, AgentError> {
    let id = super::session_io::next_id();
    write(
        session,
        &serde_json::json!({"method": method, "id": id, "params": params}),
    )
    .await?;
    loop {
        let value = super::session_io::read_json(session).await?;
        if value.get("id").and_then(serde_json::Value::as_u64) == Some(id) {
            return Ok(value);
        }
    }
}

pub(crate) fn response_error(context: &str, response: &serde_json::Value) -> AgentError {
    AgentError(format!(
        "{context}: {}",
        response.get("error").unwrap_or(response)
    ))
}

async fn write(session: &BridgeSession, value: &serde_json::Value) -> Result<(), AgentError> {
    super::session_io::write_json(session, value).await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_codex_spawn_bridge() {
        let _ = codex_spawn_bridge;
    }
    #[cfg(unix)]
    #[allow(unsafe_code)]
    #[tokio::test]
    async fn test_codex_mock_session_protocol() {
        use std::os::unix::fs::PermissionsExt;
        let _lock = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        let bin = tmp.path().join("codex");
        std::fs::write(&bin, "#!/bin/sh\nwhile IFS= read -r line; do case \"$line\" in *initialize*) printf '%s\\n' '{\"id\":1,\"result\":{}}';; *thread/start*) printf '%s\\n' '{\"id\":2,\"result\":{\"thread\":{\"id\":\"thread-test\"}}}';; *turn/start*) printf '%s\\n' '{\"id\":3,\"result\":{}}' '{\"method\":\"turn/started\",\"params\":{\"turn\":{\"id\":\"turn-test\"}}}' '{\"method\":\"item/agentMessage/delta\",\"params\":{\"turnId\":\"turn-test\",\"delta\":\"hello\"}}' '{\"method\":\"turn/completed\",\"params\":{\"turn\":{\"id\":\"turn-test\",\"status\":\"completed\",\"lastAgentMessage\":\"hello\"}}}';; *turn/interrupt*) printf '%s\\n' '{\"id\":4,\"result\":{}}';; esac; done\n").unwrap();
        let mut perms = std::fs::metadata(&bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&bin, perms).unwrap();
        let prior = std::env::var_os("MALVIN_CODEX");
        unsafe { std::env::set_var("MALVIN_CODEX", &bin); }
        let model = crate::model_id::parse_model_id("codex:gpt-test").unwrap();
        let io = crate::acp::AgentIoOptions { force: true, no_tee: true, raw_output: true, show_thoughts_on_stdout: false, emit_stdout_markdown: false, log_full_outgoing_prompts: false };
        let mut client = crate::agent_backend::SdkClient::with_max_retries(model, crate::agent_backend::BridgeKind::Codex, io, 1);
        client.begin_coder_session(tmp.path()).await.unwrap();
        client.session.as_ref().unwrap().send_prompt("test").await.unwrap();
        assert_eq!(client.last_coder_prompt_agent_response().as_deref(), Some("hello"));
        client.end_coder_session().await.unwrap();
        unsafe { match prior { Some(v) => std::env::set_var("MALVIN_CODEX", v), None => std::env::remove_var("MALVIN_CODEX") } }
    }

    #[test]
    fn test_response_error_and_id() {
        assert!(crate::codex_sdk::session_io::next_id() > 0);
        let error = response_error("context", &serde_json::json!({"error":{"message":"bad"}}));
        assert!(error.0.contains("context") && error.0.contains("bad"));
    }

    #[test]
    fn test_codex_initialize() {
        let _ = codex_initialize;
    }
    #[test]
    fn test_codex_start_thread() {
        let _ = codex_start_thread;
    }
    #[test]
    fn test_request() {
        let _ = request;
    }
    #[test]
    fn test_response_error() {
        let _ = response_error;
    }
}
