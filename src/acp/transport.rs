//! Spawning `agent acp` and JSON-RPC writes to its stdin.
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

use super::{
    ResponseTx,
    cursor_credentials::{effective_cursor_api_key, effective_cursor_auth_token},
};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io,
    path::Path,
    process::Stdio,
    sync::{atomic::{AtomicBool, AtomicU64}, Arc},
};
use tokio::{
    io::AsyncWriteExt,
    process::{Child, ChildStdin, Command},
    sync::{Mutex, oneshot},
    time::sleep,
};
use tracing::info;

const AGENT_BIN: &str = "agent";

/// Per-request wait helper for unit tests (matches [`crate::config::acp_rpc_timeout_secs_from_env`]).
#[cfg(test)]
pub(super) fn acp_rpc_timeout() -> std::time::Duration {
    std::time::Duration::from_secs(crate::config::acp_rpc_timeout_secs_from_env())
}

fn agent_program(bin_override: Option<&Path>) -> String {
    bin_override
        .map(|p| p.to_string_lossy().into_owned())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| AGENT_BIN.to_string())
}

pub(crate) fn requires_cursor_login_auth(
    explicit_api_key: Option<&str>,
    explicit_auth_token: Option<&str>,
) -> bool {
    effective_cursor_api_key(explicit_api_key).is_none()
        && effective_cursor_auth_token(explicit_auth_token).is_none()
}

pub(crate) fn build_agent_acp_command(
    cwd: &Path,
    bin_override: Option<&Path>,
    api_key: Option<&str>,
    auth_token: Option<&str>,
    george_acp_lane: Option<&str>,
    model: Option<&str>,
    force: bool,
) -> Command {
    let mut cmd = Command::new(agent_program(bin_override));
    for key in [
        "CURSOR_AUTH_TOKEN",
        "CURSOR_CONFIG_DIR",
        "HOME",
        "NO_OPEN_BROWSER",
        "XDG_CONFIG_HOME",
        "XDG_STATE_HOME",
    ] {
        if let Ok(v) = std::env::var(key)
            && !v.is_empty()
        {
            cmd.env(key, v);
        }
    }
    let api_key = effective_cursor_api_key(api_key);
    let auth_token = effective_cursor_auth_token(auth_token);
    if let Some(ref k) = api_key {
        cmd.arg("--api-key").arg(k.as_str());
        cmd.env("CURSOR_API_KEY", k.as_str());
    }
    if let Some(ref t) = auth_token {
        cmd.arg("--auth-token").arg(t.as_str());
        cmd.env("CURSOR_AUTH_TOKEN", t.as_str());
    }
    if force {
        cmd.arg("--force");
    }
    if let Some(m) = model.map(str::trim).filter(|s| !s.is_empty()) {
        cmd.arg("--model").arg(m);
    }
    cmd.arg("acp");
    cmd.env("MALVIN_WORKSPACE", cwd);
    if let Some(lane) = george_acp_lane.map(str::trim).filter(|s| !s.is_empty()) {
        cmd.env("GEORGE_ACP_LANE", lane);
    }
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .current_dir(cwd);
    cmd
}

fn executable_text_busy(err: &io::Error) -> bool {
    if err.kind() == io::ErrorKind::ExecutableFileBusy {
        return true;
    }
    #[cfg(unix)]
    {
        err.raw_os_error() == Some(26)
    }
    #[cfg(not(unix))]
    {
        let _ = err;
        false
    }
}

pub(crate) async fn spawn_agent_acp_child(cmd: &mut Command) -> Result<Child, io::Error> {
    const ATTEMPTS: u32 = 16;
    const DELAY_MS: u64 = 10;
    for attempt in 0..ATTEMPTS {
        match cmd.spawn() {
            Ok(child) => return Ok(child),
            Err(e) if executable_text_busy(&e) && attempt + 1 < ATTEMPTS => {
                sleep(std::time::Duration::from_millis(DELAY_MS)).await;
            }
            Err(e) => return Err(e),
        }
    }
    Err(io::Error::other(
        "agent acp spawn retries exhausted (internal error)",
    ))
}

pub(crate) async fn write_rpc_line(
    stdin: &Arc<Mutex<ChildStdin>>,
    line: &str,
    acp_verbose: bool,
) -> Result<(), String> {
    if acp_verbose {
        info!(
            target: "malvin::acp::io",
            direction = "out",
            line = %line,
            "acp message"
        );
    }
    let mut guard = stdin.lock().await;
    guard
        .write_all(line.as_bytes())
        .await
        .map_err(|e| format!("acp stdin write: {e}"))?;
    guard
        .write_all(b"\n")
        .await
        .map_err(|e| format!("acp stdin newline: {e}"))?;
    guard
        .flush()
        .await
        .map_err(|e| format!("acp stdin flush: {e}"))?;
    Ok(())
}

/// Shared stdio transport state for JSON-RPC to `agent acp`.
pub(crate) struct AcpStdioRpc {
    pub reader_dead: Arc<AtomicBool>,
    pub stdin: Arc<Mutex<ChildStdin>>,
    pub pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    pub acp_verbose: bool,
}

async fn rpc_request_with_correlation_id(
    io: &AcpStdioRpc,
    id: u64,
    method: &str,
    params: Value,
    rpc_timeout: std::time::Duration,
) -> Result<Value, String> {
    if io.reader_dead.load(std::sync::atomic::Ordering::SeqCst) {
        return Err("acp session is dead".into());
    }
    let (tx, rx) = oneshot::channel();
    io.pending.lock().await.insert(id, tx);
    let req = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params
    });
    let line = match serde_json::to_string(&req) {
        Ok(l) => l,
        Err(e) => {
            io.pending.lock().await.remove(&id);
            return Err(e.to_string());
        }
    };
    if let Err(e) = write_rpc_line(&io.stdin, &line, io.acp_verbose).await {
        io.pending.lock().await.remove(&id);
        return Err(e);
    }
    rpc_wait_response(&io.pending, id, rpc_timeout, rx).await
}

pub(crate) async fn rpc_request(
    io: &AcpStdioRpc,
    next_id: &Arc<AtomicU64>,
    method: &str,
    params: Value,
    rpc_timeout: std::time::Duration,
) -> Result<Value, String> {
    let id = next_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    rpc_request_with_correlation_id(io, id, method, params, rpc_timeout).await
}

pub(crate) async fn rpc_request_with_id(
    io: &AcpStdioRpc,
    id: u64,
    method: &str,
    params: Value,
    rpc_timeout: std::time::Duration,
) -> Result<Value, String> {
    rpc_request_with_correlation_id(io, id, method, params, rpc_timeout).await
}

/// Pretty line for logs / `acp-smoke` when the peer returns a JSON-RPC 2.0 `error` object.
pub(crate) fn format_jsonrpc_error(err: &Value) -> String {
    let Some(obj) = err.as_object() else {
        return err.to_string();
    };
    let code = obj
        .get("code")
        .map(|c| c.to_string())
        .unwrap_or_else(|| "null".to_string());
    let msg = obj
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("");
    let data_detail = obj
        .get("data")
        .and_then(|d| d.get("message"))
        .and_then(|m| m.as_str())
        .or_else(|| obj.get("data").and_then(Value::as_str));
    let mut parts = vec![format!("code={code}"), format!("message={msg:?}")];
    if let Some(d) = data_detail {
        parts.push(format!("detail={d:?}"));
    }
    parts.join("; ")
}

async fn rpc_wait_response(
    pending: &Arc<Mutex<HashMap<u64, ResponseTx>>>,
    id: u64,
    rpc_timeout: std::time::Duration,
    rx: oneshot::Receiver<Result<Value, String>>,
) -> Result<Value, String> {
    match tokio::time::timeout(rpc_timeout, rx).await {
        Ok(ready) => ready.map_err(|_| "acp request canceled (session dropped)".to_string())?,
        Err(_) => {
            pending.lock().await.remove(&id);
            Err("acp RPC timed out".into())
        }
    }
}

pub(crate) async fn handshake_inner(
    io: &AcpStdioRpc,
    next_id: &Arc<AtomicU64>,
    cwd: &Path,
    rpc_timeout: std::time::Duration,
    require_cursor_login_auth: bool,
) -> Result<String, String> {
    let init = json!({
        "protocolVersion": 1,
        "clientCapabilities": {
            "fs": { "readTextFile": false, "writeTextFile": false },
            "terminal": false
        },
        "clientInfo": { "name": "malvin", "version": env!("CARGO_PKG_VERSION") }
    });
    let _ = rpc_request(io, next_id, "initialize", init, rpc_timeout)
        .await
        .map_err(|e| format!("ACP `initialize` failed: {e}"))?;
    if require_cursor_login_auth {
        let _ = rpc_request(
            io,
            next_id,
            "authenticate",
            json!({ "methodId": "cursor_login" }),
            rpc_timeout,
        )
        .await
        .map_err(|e| format!("ACP `authenticate` (methodId=cursor_login) failed: {e}"))?;
    }
    let new_params = json!({
        "cwd": cwd.to_string_lossy(),
        "mcpServers": []
    });
    let res = rpc_request(io, next_id, "session/new", new_params, rpc_timeout)
        .await
        .map_err(|e| format!("ACP `session/new` failed: {e}"))?;
    let sid = res
        .get("sessionId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "session/new missing sessionId".to_string())?;
    Ok(sid.to_string())
}

#[cfg(test)]
mod tests {
    #![allow(unsafe_code)]

    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use tokio::io::AsyncReadExt;

    fn clear_cursor_env_for_test() {
        unsafe {
            std::env::remove_var("CURSOR_API_KEY");
            std::env::remove_var("CURSOR_AUTH_TOKEN");
        }
    }

    #[test]
    fn test_acp_rpc_timeout_parsing() {
        let _g = crate::test_utils::test_env_lock();
        unsafe {
            std::env::remove_var("MALVIN_ACP_RPC_TIMEOUT_SECS");
        }
        assert_eq!(
            super::acp_rpc_timeout(),
            std::time::Duration::from_secs(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS)
        );
        unsafe {
            std::env::set_var("MALVIN_ACP_RPC_TIMEOUT_SECS", "5");
        }
        assert_eq!(
            super::acp_rpc_timeout(),
            std::time::Duration::from_secs(5)
        );
        unsafe {
            std::env::set_var("MALVIN_ACP_RPC_TIMEOUT_SECS", "0");
        }
        assert_eq!(
            super::acp_rpc_timeout(),
            std::time::Duration::from_secs(1)
        );
        unsafe {
            std::env::remove_var("MALVIN_ACP_RPC_TIMEOUT_SECS");
        }
    }

    #[test]
    fn executable_text_busy_matches_error_kind_and_unix_etxtbsy() {
        use std::io::{Error, ErrorKind};

        assert!(super::executable_text_busy(&Error::new(
            ErrorKind::ExecutableFileBusy,
            "busy"
        )));
        assert!(!super::executable_text_busy(&Error::new(
            ErrorKind::NotFound,
            "no"
        )));
        #[cfg(unix)]
        assert!(super::executable_text_busy(&Error::from_raw_os_error(26)));
    }

    fn command_args(cmd: &Command) -> Vec<String> {
        cmd.as_std()
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    fn command_env_value(cmd: &Command, key: &str) -> Option<String> {
        cmd.as_std()
            .get_envs()
            .find(|(name, _)| *name == key)
            .and_then(|(_, value)| value.map(|v| v.to_string_lossy().into_owned()))
    }

    fn assert_arg_value(args: &[String], flag: &str, expected: Option<&str>) {
        if let Some(value) = expected {
            assert!(
                args.windows(2).any(|pair| pair[0] == flag && pair[1] == value),
                "expected `{flag} {value}` in args: {args:?}"
            );
        } else {
            assert!(
                !args.iter().any(|arg| arg == flag),
                "did not expect `{flag}` in args: {args:?}"
            );
        }
    }

    fn assert_cursor_credentials_forwarding(
        cmd: &Command,
        expected_key: Option<&str>,
        expected_token: Option<&str>,
    ) {
        let args = command_args(cmd);
        assert_arg_value(&args, "--api-key", expected_key);
        assert_arg_value(&args, "--auth-token", expected_token);
        assert_eq!(command_env_value(cmd, "CURSOR_API_KEY").as_deref(), expected_key);
        assert_eq!(
            command_env_value(cmd, "CURSOR_AUTH_TOKEN").as_deref(),
            expected_token
        );
    }

    #[test]
    fn test_cursor_credentials_forwards_key_and_token() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            Some("key-a"),
            Some("tok-b"),
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, Some("key-a"), Some("tok-b"));
    }

    #[test]
    fn test_cursor_credentials_key_only() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            Some("k-only"),
            None,
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, Some("k-only"), None);
    }

    #[test]
    fn test_cursor_credentials_explicit_none_uses_process_env_api_key() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        unsafe {
            std::env::set_var("CURSOR_API_KEY", "key-from-process-env");
        }
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            None,
            None,
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, Some("key-from-process-env"), None);
        clear_cursor_env_for_test();
    }

    #[test]
    fn test_cursor_credentials_explicit_none_uses_process_env_auth_token() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        unsafe {
            std::env::set_var("CURSOR_AUTH_TOKEN", "tok-from-process-env");
        }
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            None,
            None,
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, None, Some("tok-from-process-env"));
        clear_cursor_env_for_test();
    }

    #[test]
    fn test_cursor_credentials_explicit_api_key_overrides_process_env() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        unsafe {
            std::env::set_var("CURSOR_API_KEY", "from-env");
        }
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            Some("explicit-wins"),
            None,
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, Some("explicit-wins"), None);
        clear_cursor_env_for_test();
    }

    #[test]
    fn test_cursor_credentials_explicit_auth_token_overrides_process_env() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        unsafe {
            std::env::set_var("CURSOR_AUTH_TOKEN", "from-env");
        }
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            None,
            Some("explicit-tok"),
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, None, Some("explicit-tok"));
        clear_cursor_env_for_test();
    }

    /// Neither explicit nor `CURSOR_*` env: no credentials forwarded.
    #[test]
    fn test_cursor_credentials_absent_process_env_and_no_explicit() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            None,
            None,
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, None, None);
    }

    /// Empty explicit key skips to `CURSOR_API_KEY` from the process when set.
    #[test]
    fn test_cursor_credentials_empty_explicit_key_falls_back_to_process_env() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        unsafe {
            std::env::set_var("CURSOR_API_KEY", "env-after-empty-explicit");
        }
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            Some(""),
            None,
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, Some("env-after-empty-explicit"), None);
        clear_cursor_env_for_test();
    }

    /// Empty `CURSOR_API_KEY` in the environment is ignored (treated as unset).
    #[test]
    fn test_cursor_credentials_process_env_empty_api_key_ignored() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        unsafe {
            std::env::set_var("CURSOR_API_KEY", "");
        }
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            None,
            None,
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, None, None);
        clear_cursor_env_for_test();
    }

    /// Empty explicit token skips to `CURSOR_AUTH_TOKEN` from the process when set.
    #[test]
    fn test_cursor_credentials_empty_explicit_token_falls_back_to_process_env() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        unsafe {
            std::env::set_var("CURSOR_AUTH_TOKEN", "env-after-empty-explicit-tok");
        }
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            None,
            Some(""),
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, None, Some("env-after-empty-explicit-tok"));
        clear_cursor_env_for_test();
    }

    /// Empty `CURSOR_AUTH_TOKEN` in the environment is ignored.
    #[test]
    fn test_cursor_credentials_process_env_empty_auth_token_ignored() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        unsafe {
            std::env::set_var("CURSOR_AUTH_TOKEN", "");
        }
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            None,
            None,
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, None, None);
        clear_cursor_env_for_test();
    }

    #[test]
    fn test_cursor_credentials_token_only() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            None,
            Some("t-only"),
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, None, Some("t-only"));
    }

    #[test]
    fn test_cursor_credentials_empty_strings_skipped() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            Some(""),
            Some(""),
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, None, None);
    }

    #[test]
    fn test_cursor_credentials_skips_empty_key_only() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            Some(""),
            Some("tok2"),
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, None, Some("tok2"));
    }

    #[test]
    fn test_cursor_credentials_skips_empty_token_only() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        let tmp = tempfile::tempdir().unwrap();
        let cmd = build_agent_acp_command(
            tmp.path(),
            Some(Path::new("/bin/true")),
            Some("k2"),
            Some(""),
            None,
            None,
            false,
        );
        assert_cursor_credentials_forwarding(&cmd, Some("k2"), None);
    }

    #[tokio::test]
    async fn test_write_rpc_line_fails_after_child_stdin_closed() {
        use std::time::Duration;

        let mut child = Command::new("sleep")
            .arg("60")
            .stdin(Stdio::piped())
            .spawn()
            .expect("sleep");
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let _ = child.kill().await;
        let _ = child.wait().await;

        // Kernel/async reactor may observe a closed read-end slightly after `wait` returns; poll
        // until `write_rpc_line` sees `EPIPE` / broken pipe (or time out).
        let mut last = Ok(());
        for _ in 0..100 {
            last = write_rpc_line(&stdin, r#"{"x":1}"#, false).await;
            if last.is_err() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        panic!("expected stdin write to fail after child exit (broken pipe), last={last:?}");
    }

    #[test]
    fn format_jsonrpc_error_pretty_prints_cursor_style() {
        let err = json!({
            "code": -32602,
            "message": "Invalid params",
            "data": {"message": "Failed to open browser for login."}
        });
        let s = format_jsonrpc_error(&err);
        assert!(s.contains("32602"), "{s}");
        assert!(s.contains("Invalid params"), "{s}");
        assert!(s.contains("Failed to open browser"), "{s}");
    }

    #[test]
    fn format_jsonrpc_error_falls_back_for_non_object() {
        assert_eq!(format_jsonrpc_error(&json!("plain")), "\"plain\"");
    }

    #[test]
    fn test_agent_program_prefers_nonempty_override() {
        let p = Path::new("/tmp/mock-agent-override");
        assert!(agent_program(Some(p)).contains("mock-agent-override"));
        assert_eq!(agent_program(Some(Path::new(""))), AGENT_BIN);
        assert_eq!(agent_program(None), AGENT_BIN);
    }

    #[test]
    fn requires_cursor_login_auth_skips_login_when_process_credentials_exist() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        unsafe {
            std::env::set_var("CURSOR_API_KEY", "key-from-env");
        }
        assert!(!requires_cursor_login_auth(None, None));
        clear_cursor_env_for_test();
    }

    async fn write_bad_session_new_mock(bin: &Path) {
        let script = format!(
            "#!/usr/bin/env node\n{}",
            crate::test_utils::ACP_MOCK_JSONRPC_LOOP_JS
        )
        .replace(
            "result: { sessionId: 't1' }",
            "result: { wrongKey: 't1' }",
        );
        tokio::fs::write(bin, script.as_bytes()).await.unwrap();
        let mut perms = std::fs::metadata(bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(bin, perms).unwrap();
        crate::test_utils::sync_test_executable(bin);
    }

    async fn write_authenticate_rejected_but_session_new_ok_mock(bin: &Path) {
        let script = r#"#!/usr/bin/env node
const readline = require('readline');
const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
rl.on('line', (line) => {
  line = line.trim();
  if (!line) return;
  let msg;
  try { msg = JSON.parse(line); } catch (e) { return; }
  const mid = msg.method;
  const rid = msg.id;
  if (mid === 'initialize') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'authenticate') {
    console.log(JSON.stringify({
      jsonrpc: '2.0',
      id: rid,
      error: {
        code: -32602,
        message: 'Invalid params',
        data: { message: 'Failed to open browser for login.' },
      },
    }));
  } else if (mid === 'session/new') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { sessionId: 't1' } }));
  } else if (rid != null) {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});
"#;
        tokio::fs::write(bin, script.as_bytes()).await.unwrap();
        let mut perms = std::fs::metadata(bin).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(bin, perms).unwrap();
        crate::test_utils::sync_test_executable(bin);
    }

    fn spawn_test_reader_loop(
        stdout: tokio::process::ChildStdout,
        pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
        stdin: Arc<Mutex<ChildStdin>>,
        reader_dead: Arc<AtomicBool>,
    ) {
        let trace_writer: Arc<Mutex<Option<tokio::fs::File>>> = Arc::new(Mutex::new(None));
        let pending_r = pending.clone();
        let stdin_r = stdin.clone();
        let dead_r = reader_dead.clone();
        tokio::spawn(async move {
            crate::acp::reader::reader_loop(
                stdout,
                pending_r,
                stdin_r,
                dead_r,
                trace_writer,
                None,
                false,
            )
            .await;
        });
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn test_handshake_hits_session_new_error_path() {
        let _guard = crate::test_utils::test_env_lock();
        let tmp = tempfile::tempdir().unwrap();
        let bin = tmp.path().join(format!(
            "bad-session-mock-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .subsec_nanos()
        ));
        write_bad_session_new_mock(&bin).await;

        let mut cmd = build_agent_acp_command(tmp.path(), Some(&bin), Some(""), Some(""), None, None, false);
        let mut child = super::spawn_agent_acp_child(&mut cmd).await.expect("spawn");
        let stdin = Arc::new(Mutex::new(child.stdin.take().unwrap()));
        let stdout = child.stdout.take().unwrap();

        let pending = Arc::new(Mutex::new(HashMap::new()));
        let reader_dead = Arc::new(AtomicBool::new(false));
        let next_id = Arc::new(AtomicU64::new(1));

        spawn_test_reader_loop(stdout, pending.clone(), stdin.clone(), reader_dead.clone());

        let io = AcpStdioRpc {
            reader_dead,
            stdin,
            pending,
            acp_verbose: false,
        };
        let err = handshake_inner(&io, &next_id, tmp.path(), acp_rpc_timeout(), true)
            .await
        .unwrap_err();
        assert!(err.contains("sessionId"));

        let _ = child.kill().await;
        let _ = child.wait().await;
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn handshake_can_skip_cursor_login_when_api_key_mode_is_used() {
        let _guard = crate::test_utils::test_env_lock();
        clear_cursor_env_for_test();
        unsafe {
            std::env::set_var("CURSOR_API_KEY", "env-key");
        }
        let tmp = tempfile::tempdir().unwrap();
        let bin = tmp.path().join("auth-rejected-session-ok");
        write_authenticate_rejected_but_session_new_ok_mock(&bin).await;

        let mut cmd = build_agent_acp_command(tmp.path(), Some(&bin), None, None, None, None, false);
        let mut child = super::spawn_agent_acp_child(&mut cmd).await.expect("spawn");
        let stdin = Arc::new(Mutex::new(child.stdin.take().unwrap()));
        let stdout = child.stdout.take().unwrap();

        let pending = Arc::new(Mutex::new(HashMap::new()));
        let reader_dead = Arc::new(AtomicBool::new(false));
        let next_id = Arc::new(AtomicU64::new(1));

        spawn_test_reader_loop(stdout, pending.clone(), stdin.clone(), reader_dead.clone());

        let io = AcpStdioRpc {
            reader_dead,
            stdin,
            pending,
            acp_verbose: false,
        };
        let sid = handshake_inner(&io, &next_id, tmp.path(), acp_rpc_timeout(), false)
            .await
            .expect("session/new should work without cursor_login authenticate");
        assert_eq!(sid, "t1");

        clear_cursor_env_for_test();
        let _ = child.kill().await;
        let _ = child.wait().await;
    }

    #[tokio::test]
    async fn test_rpc_cancel_when_pending_sender_dropped() {
        let mut child = Command::new("sleep")
            .arg("60")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("sleep");
        let stdin = Arc::new(Mutex::new(child.stdin.take().unwrap()));
        let mut stdout = child.stdout.take().unwrap();
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let reader_dead = Arc::new(AtomicBool::new(false));
        let next_id = Arc::new(AtomicU64::new(1));

        let io = AcpStdioRpc {
            reader_dead,
            stdin: stdin.clone(),
            pending: pending.clone(),
            acp_verbose: false,
        };
        let drain = tokio::spawn(async move {
            let mut buf = vec![0u8; 256];
            loop {
                if stdout.read(&mut buf).await.unwrap_or(0) == 0 {
                    break;
                }
            }
        });

        let send = tokio::spawn(async move {
            let r = rpc_request(&io, &next_id, "nope", json!({}), acp_rpc_timeout()).await;
            let e = r.unwrap_err();
            assert!(e.contains("canceled") || e.contains("session"), "{e}");
        });

        for _ in 0..20 {
            if !pending.lock().await.is_empty() {
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
        pending.lock().await.clear();

        let _ = child.kill().await;
        let _ = child.wait().await;
        let _ = send.await;
        let _ = drain.await;
    }

    /// Regression: `rpc_request` must not leave `pending` entries when the request never leaves
    /// the process (e.g. stdin closed before `write_rpc_line`).
    #[tokio::test]
    async fn test_rpc_request_does_not_leak_pending_after_write_failure() {
        let mut child = Command::new("true")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("true");
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let mut stdout = child.stdout.take().expect("stdout");
        let drain = tokio::spawn(async move {
            let mut buf = vec![0u8; 64];
            while stdout.read(&mut buf).await.unwrap_or(0) > 0 {}
        });
        let _ = child.wait().await;

        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let reader_dead = Arc::new(AtomicBool::new(false));
        let next_id = Arc::new(AtomicU64::new(1));

        let io = AcpStdioRpc {
            reader_dead,
            stdin,
            pending,
            acp_verbose: false,
        };
        let err = rpc_request(&io, &next_id, "nope", json!({}), acp_rpc_timeout())
            .await
        .expect_err("stdin write after child exit should fail");

        assert!(!err.is_empty(), "{err}");
        assert!(
            io.pending.lock().await.is_empty(),
            "pending should be cleared when write fails; leaked ids: {:?}",
            io.pending.lock().await.keys().copied().collect::<Vec<_>>()
        );

        let _ = drain.await;
    }

    #[tokio::test]
    async fn rpc_request_with_correlation_id_times_out_when_stdout_silent() {
        let mut child = Command::new("sleep")
            .arg("15")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("sleep");
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let mut stdout = child.stdout.take().expect("stdout");
        tokio::spawn(async move {
            let mut buf = [0u8; 256];
            while stdout.read(&mut buf).await.unwrap_or(0) > 0 {}
        });
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let reader_dead = Arc::new(AtomicBool::new(false));
        let io = AcpStdioRpc {
            reader_dead,
            stdin,
            pending,
            acp_verbose: false,
        };
        let err = rpc_request_with_correlation_id(
            &io,
            3,
            "unanswered",
            json!({}),
            std::time::Duration::from_millis(25),
        )
        .await
        .expect_err("peer never responds");
        assert!(
            err.contains("timed out") || err.contains("acp RPC"),
            "unexpected err: {err}"
        );
        let _ = child.kill().await;
        let _ = child.wait().await;
    }

    #[tokio::test]
    async fn rpc_request_with_id_errors_when_reader_dead() {
        let reader_dead = Arc::new(AtomicBool::new(true));
        let mut child = Command::new("sleep")
            .arg("2")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("sleep");
        let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
        let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
        let io = AcpStdioRpc {
            reader_dead,
            stdin,
            pending,
            acp_verbose: false,
        };
        let err = rpc_request_with_id(
            &io,
            7,
            "nope",
            json!({}),
            std::time::Duration::from_millis(500),
        )
        .await
        .expect_err("reader flagged dead");
        assert!(err.contains("dead"), "{err}");
        let _ = child.kill().await;
        let _ = child.wait().await;
    }
}
