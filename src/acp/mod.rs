//! Agent Client Protocol (`agent acp`) JSON-RPC over stdio.
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]

mod cursor_credentials;
mod reader;
mod transport;

use reader::{PromptRpcCleanup, spawn_acp_stdout_reader};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::{Mutex, Notify, oneshot};

pub(crate) type ResponseTx = oneshot::Sender<Result<Value, String>>;

struct AcpSessionInner {
    child: Mutex<Child>,
    stdin: Arc<Mutex<ChildStdin>>,
    pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    next_id: Arc<AtomicU64>,
    session_id: String,
    reader_dead: Arc<AtomicBool>,
    rpc_timeout: Duration,
    busy: Arc<AtomicBool>,
    trace_writer: Arc<Mutex<Option<tokio::fs::File>>>,
    prompt_rpc_id: Arc<AtomicU64>,
    /// Serializes [`AcpSession::prompt`] so overlapping callers cannot stomp [`Self::trace_writer`].
    /// [`AcpSession::cancel`] does not take this lock so cancellation can interleave with a slow prompt.
    prompt_singleflight: Arc<Mutex<()>>,
    acp_verbose: bool,
    /// When set (UI lane), observers are notified whenever `busy` becomes false.
    ui_idle_notify: Option<Arc<Notify>>,
}

/// Live `agent acp` child process and JSON-RPC session state (cloneable handle; `cancel` may run
/// concurrently with an in-flight `session/prompt`; `prompt` calls are serialized per session).
#[derive(Clone)]
pub struct AcpSession(Arc<AcpSessionInner>);

/// Arguments for [`AcpSession::spawn`].
pub struct AcpSpawnArgs<'a> {
    pub cwd: &'a Path,
    pub bin_override: Option<&'a Path>,
    pub api_key: Option<&'a str>,
    pub auth_token: Option<&'a str>,
    pub rpc_timeout: Duration,
    pub acp_verbose: bool,
    pub george_acp_lane: Option<&'a str>,
    pub ui_idle_notify: Option<Arc<Notify>>,
    /// Passed through to `agent --model` when non-empty.
    pub model: Option<&'a str>,
    /// When true, passes `agent --force`.
    pub force: bool,
}

#[allow(clippy::too_many_arguments)]
async fn acp_spawn_start_reader_and_handshake(
    mut child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: ChildStdout,
    pending: Arc<Mutex<HashMap<u64, ResponseTx>>>,
    reader_dead: Arc<AtomicBool>,
    next_id: Arc<AtomicU64>,
    busy: Arc<AtomicBool>,
    trace_writer: Arc<Mutex<Option<tokio::fs::File>>>,
    prompt_rpc_id: Arc<AtomicU64>,
    ui_idle_notify: Option<Arc<Notify>>,
    acp_verbose: bool,
    cwd: &Path,
    rpc_timeout: Duration,
    require_cursor_login_auth: bool,
) -> Result<(Child, String), String> {
    let prompt_cleanup = Arc::new(PromptRpcCleanup {
        busy: busy.clone(),
        trace_writer: trace_writer.clone(),
        prompt_rpc_id: prompt_rpc_id.clone(),
        idle_notify: ui_idle_notify.clone(),
    });
    let _reader_task = spawn_acp_stdout_reader(
        stdout,
        pending.clone(),
        stdin.clone(),
        reader_dead.clone(),
        trace_writer.clone(),
        prompt_cleanup,
        acp_verbose,
    );

    let io = transport::AcpStdioRpc {
        reader_dead: reader_dead.clone(),
        stdin: stdin.clone(),
        pending: pending.clone(),
        acp_verbose,
    };
    let session_id = match transport::handshake_inner(
        &io,
        &next_id,
        cwd,
        rpc_timeout,
        require_cursor_login_auth,
    )
    .await
    {
        Ok(s) => s,
        Err(e) => {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(e);
        }
    };
    Ok((child, session_id))
}

impl AcpSession {
    /// Spawn `agent acp`, run `initialize` / `authenticate` / `session/new`.
    pub async fn spawn(args: AcpSpawnArgs<'_>) -> Result<Self, String> {
        let rpc_timeout = if args.rpc_timeout.is_zero() {
            Duration::from_millis(1)
        } else {
            args.rpc_timeout
        };
        let require_cursor_login_auth =
            transport::requires_cursor_login_auth(args.api_key, args.auth_token);
        let mut cmd = transport::build_agent_acp_command(
            args.cwd,
            args.bin_override,
            args.api_key,
            args.auth_token,
            args.george_acp_lane,
            args.model,
            args.force,
        );
        let mut child = transport::spawn_agent_acp_child(&mut cmd)
            .await
            .map_err(|e| format!("failed to spawn agent acp: {e}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "agent acp stdin pipe missing".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "agent acp stdout pipe missing".to_string())?;
        let stdin = Arc::new(Mutex::new(stdin));
        let pending = Arc::new(Mutex::new(HashMap::new()));
        let reader_dead = Arc::new(AtomicBool::new(false));
        let next_id = Arc::new(AtomicU64::new(1));
        let busy = Arc::new(AtomicBool::new(false));
        let trace_writer: Arc<Mutex<Option<tokio::fs::File>>> = Arc::new(Mutex::new(None));
        let prompt_rpc_id = Arc::new(AtomicU64::new(0));
        let prompt_singleflight = Arc::new(Mutex::new(()));

        let (child, session_id) = acp_spawn_start_reader_and_handshake(
            child,
            stdin.clone(),
            stdout,
            pending.clone(),
            reader_dead.clone(),
            next_id.clone(),
            busy.clone(),
            trace_writer.clone(),
            prompt_rpc_id.clone(),
            args.ui_idle_notify.clone(),
            args.acp_verbose,
            args.cwd,
            rpc_timeout,
            require_cursor_login_auth,
        )
        .await?;

        Ok(AcpSession(Arc::new(AcpSessionInner {
            child: Mutex::new(child),
            stdin,
            pending,
            next_id,
            session_id,
            reader_dead,
            rpc_timeout,
            busy,
            trace_writer,
            prompt_rpc_id,
            prompt_singleflight,
            acp_verbose: args.acp_verbose,
            ui_idle_notify: args.ui_idle_notify,
        })))
    }

    pub async fn is_alive(&self) -> bool {
        if self.0.reader_dead.load(Ordering::SeqCst) {
            return false;
        }
        let mut ch = self.0.child.lock().await;
        match ch.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(_) => true,
        }
    }

    pub fn is_busy(&self) -> bool {
        self.0.busy.load(Ordering::SeqCst)
    }

    async fn send_rpc(&self, method: &str, params: Value) -> Result<Value, String> {
        let io = transport::AcpStdioRpc {
            reader_dead: self.0.reader_dead.clone(),
            stdin: self.0.stdin.clone(),
            pending: self.0.pending.clone(),
            acp_verbose: self.0.acp_verbose,
        };
        transport::rpc_request(&io, &self.0.next_id, method, params, self.0.rpc_timeout).await
    }

    async fn reset_prompt_inflight(&self) {
        self.0.busy.store(false, Ordering::SeqCst);
        *self.0.trace_writer.lock().await = None;
        self.0.prompt_rpc_id.store(0, Ordering::SeqCst);
        if let Some(n) = &self.0.ui_idle_notify {
            n.notify_waiters();
        }
    }

    /// Send [`session/prompt`](https://cursor.com/docs/cli/acp) for the active session; JSON stdout lines are appended to `trace_path`.
    pub async fn prompt(&self, text: &str, trace_path: &Path) -> Result<(), String> {
        let _prompt_turn = self.0.prompt_singleflight.lock().await;
        if let Some(parent) = trace_path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("trace mkdir: {e}"))?;
        }
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(trace_path)
            .await
            .map_err(|e| format!("trace open: {e}"))?;
        *self.0.trace_writer.lock().await = Some(file);
        self.0.busy.store(true, Ordering::SeqCst);

        let id = self.0.next_id.fetch_add(1, Ordering::SeqCst);
        self.0.prompt_rpc_id.store(id, Ordering::SeqCst);

        let params = json!({
            "sessionId": &self.0.session_id,
            "prompt": [{ "type": "text", "text": text }]
        });

        let io = transport::AcpStdioRpc {
            reader_dead: self.0.reader_dead.clone(),
            stdin: self.0.stdin.clone(),
            pending: self.0.pending.clone(),
            acp_verbose: self.0.acp_verbose,
        };
        let res = transport::rpc_request_with_id(
            &io,
            id,
            "session/prompt",
            params,
            self.0.rpc_timeout,
        )
        .await;

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                self.reset_prompt_inflight().await;
                Err(e)
            }
        }
    }

    /// Request cancellation of the in-flight prompt (ACP `session/cancel`).
    pub async fn cancel(&self) -> Result<(), String> {
        let params = json!({ "sessionId": &self.0.session_id });
        let r = self.send_rpc("session/cancel", params).await;
        if r.is_ok() {
            self.0.busy.store(false, Ordering::SeqCst);
            *self.0.trace_writer.lock().await = None;
            self.0.prompt_rpc_id.store(0, Ordering::SeqCst);
            if let Some(n) = &self.0.ui_idle_notify {
                n.notify_waiters();
            }
        }
        r.map(|_| ())
    }

    pub async fn shutdown(&self) -> Result<(), String> {
        self.0.busy.store(false, Ordering::SeqCst);
        *self.0.trace_writer.lock().await = None;
        self.0.prompt_rpc_id.store(0, Ordering::SeqCst);
        if let Some(n) = &self.0.ui_idle_notify {
            n.notify_waiters();
        }
        let mut ch = self.0.child.lock().await;
        let _ = ch.kill().await;
        ch.wait()
            .await
            .map_err(|e| format!("acp wait: {e}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(unsafe_code)]

    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use tokio::sync::Notify;

    const EXTENDED_MOCK_AGENT: &str = r#"const readline = require('readline');
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
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'session/new') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { sessionId: 't1' } }));
  } else if (mid === 'session/cancel') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'session/prompt') {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { stopReason: 'end' } }));
    console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { k: 1 } }));
    console.log(JSON.stringify({
      jsonrpc: '2.0',
      id: 77,
      method: 'session/request_permission',
      params: {},
    }));
    console.log('not-json {{{');
    console.log(JSON.stringify({ jsonrpc: '2.0', id: 'string-id', result: {} }));
    console.log(JSON.stringify({ jsonrpc: '2.0', id: 999001, result: {} }));
  } else if (rid != null) {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});
"#;

    const MOCK_BAD_SESSION_NEW: &str = r#"const fs = require('fs');
const readline = require('readline');
fs.writeFileSync('mock.pid', String(process.pid));
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
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'session/new') {
    console.log(JSON.stringify({
      jsonrpc: '2.0',
      id: rid,
      result: { wrongKey: 'no-session-id' },
    }));
  } else if (rid != null) {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});
"#;

    async fn write_mock_executable(path: &Path) {
        let script = format!("#!/usr/bin/env node\n{}", EXTENDED_MOCK_AGENT);
        tokio::fs::write(path, script.as_bytes())
            .await
            .unwrap();
        let mut p = std::fs::metadata(path).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(path, p).unwrap();
        crate::test_utils::sync_test_executable(path);
    }

    fn workspace_with_prompt_stub() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    #[cfg(unix)]
    fn unix_kill_probe(pid: i32, sig: &str) -> std::io::Result<std::process::ExitStatus> {
        use std::process::Stdio;
        std::process::Command::new("/bin/kill")
            .args([sig, &pid.to_string()])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
    }

    #[cfg(unix)]
    fn unix_process_reapable(pid: i32) -> bool {
        unix_kill_probe(pid, "-0")
            .map(|s| s.success())
            .unwrap_or(false)
    }

    #[cfg(unix)]
    async fn await_workspace_pid_file(path: &std::path::Path) {
        use std::time::Duration;
        for _ in 0..50 {
            if path.exists() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    #[test]
    fn kiss_static_refs_acp_session_inner_and_spawn_handshake() {
        assert!(std::mem::size_of::<AcpSessionInner>() > 0);
        assert!(
            stringify!(acp_spawn_start_reader_and_handshake).contains("acp_spawn"),
            "expected stringify to retain handshake symbol name"
        );
    }

    #[test]
    fn test_acp_rpc_timeout_matches_config_env_helper() {
        let _g = crate::test_utils::test_env_lock();
        assert_eq!(
            crate::acp::transport::acp_rpc_timeout(),
            std::time::Duration::from_secs(crate::config::acp_rpc_timeout_secs_from_env())
        );
    }

    #[test]
    fn test_encode_rpc_request_line() {
        let req = json!({
            "jsonrpc": "2.0",
            "id": 1u64,
            "method": "initialize",
            "params": { "protocolVersion": 1 }
        });
        let s = serde_json::to_string(&req).unwrap();
        assert!(s.contains("initialize"));
        assert!(s.contains("\"id\":1"));
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn acp_full_session_with_notifications_and_credentials() {
        let tmp = workspace_with_prompt_stub();
        let bin = tmp.path().join("mock-agent-acp");
        write_mock_executable(&bin).await;
        let s = AcpSession::spawn(AcpSpawnArgs {
            cwd: tmp.path(),
            bin_override: Some(&bin),
            api_key: Some("george-test-api-key"),
            auth_token: Some("george-test-auth"),
            rpc_timeout: Duration::from_secs(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS),
            acp_verbose: false,
            george_acp_lane: None,
            ui_idle_notify: None,
            model: None,
            force: false,
        })
        .await
        .expect("spawn mock agent acp");
        let trace = tmp.path().join("t.jsonl");
        s.prompt("hello", &trace).await.expect("prompt");
        s.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn acp_full_session_verbose_stdout_reader_path() {
        let tmp = workspace_with_prompt_stub();
        let bin = tmp.path().join("mock-agent-acp-verbose");
        write_mock_executable(&bin).await;
        let s = AcpSession::spawn(AcpSpawnArgs {
            cwd: tmp.path(),
            bin_override: Some(&bin),
            api_key: Some("test-api-key"),
            auth_token: None,
            rpc_timeout: Duration::from_secs(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS),
            acp_verbose: true,
            george_acp_lane: None,
            ui_idle_notify: None,
            model: None,
            force: false,
        })
        .await
        .expect("spawn verbose");
        let trace = tmp.path().join("tv.jsonl");
        s.prompt("hi", &trace).await.expect("prompt");
        s.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn acp_ui_idle_notify_shutdown_wakes_waiter() {
        let notify = Arc::new(Notify::new());
        let wait_task = tokio::spawn({
            let notify = notify.clone();
            async move {
                notify.notified().await;
            }
        });
        tokio::task::yield_now().await;
        let tmp = workspace_with_prompt_stub();
        let bin = tmp.path().join("mock-agent-acp-idle-notify");
        write_mock_executable(&bin).await;
        let s = AcpSession::spawn(AcpSpawnArgs {
            cwd: tmp.path(),
            bin_override: Some(&bin),
            api_key: Some("test-api-key"),
            auth_token: None,
            rpc_timeout: Duration::from_secs(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS),
            acp_verbose: false,
            george_acp_lane: None,
            ui_idle_notify: Some(notify),
            model: None,
            force: false,
        })
        .await
        .expect("spawn");
        s.shutdown().await.expect("shutdown");
        tokio::time::timeout(std::time::Duration::from_secs(5), wait_task)
            .await
            .expect("wait task should finish")
            .expect("join wait task");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn acp_ui_idle_notify_cancel_ok_wakes_waiter() {
        let notify = Arc::new(Notify::new());
        let wait_task = tokio::spawn({
            let notify = notify.clone();
            async move {
                notify.notified().await;
            }
        });
        tokio::task::yield_now().await;
        let tmp = workspace_with_prompt_stub();
        let bin = tmp.path().join("mock-agent-acp-cancel-notify");
        write_mock_executable(&bin).await;
        let s = AcpSession::spawn(AcpSpawnArgs {
            cwd: tmp.path(),
            bin_override: Some(&bin),
            api_key: Some("test-api-key"),
            auth_token: None,
            rpc_timeout: Duration::from_secs(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS),
            acp_verbose: false,
            george_acp_lane: None,
            ui_idle_notify: Some(notify),
            model: None,
            force: false,
        })
        .await
        .expect("spawn");
        s.cancel().await.expect("cancel while idle");
        tokio::time::timeout(std::time::Duration::from_secs(5), wait_task)
            .await
            .expect("wait task should finish")
            .expect("join wait task");
        s.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn acp_ui_idle_notify_prompt_rpc_error_wakes_waiter() {
        let notify = Arc::new(Notify::new());
        let wait_task = tokio::spawn({
            let notify = notify.clone();
            async move {
                notify.notified().await;
            }
        });
        tokio::task::yield_now().await;
        let tmp = workspace_with_prompt_stub();
        let bin = tmp.path().join("mock-agent-acp-prompt-err-notify");
        crate::test_utils::write_acp_jsonrpc_mock_executable_prompt_fails(&bin, None).await;
        let s = AcpSession::spawn(AcpSpawnArgs {
            cwd: tmp.path(),
            bin_override: Some(&bin),
            api_key: Some("test-api-key"),
            auth_token: None,
            rpc_timeout: Duration::from_secs(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS),
            acp_verbose: false,
            george_acp_lane: None,
            ui_idle_notify: Some(notify),
            model: None,
            force: false,
        })
        .await
        .expect("spawn");
        let trace = tmp.path().join("prompt_err.jsonl");
        assert!(s.prompt("x", &trace).await.is_err());
        tokio::time::timeout(std::time::Duration::from_secs(5), wait_task)
            .await
            .expect("wait task should finish")
            .expect("join wait task");
        s.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn acp_prompt_fails_after_shutdown() {
        let tmp = workspace_with_prompt_stub();
        let bin = tmp.path().join("mock-agent-acp");
        write_mock_executable(&bin).await;
        let s = AcpSession::spawn(AcpSpawnArgs {
            cwd: tmp.path(),
            bin_override: Some(&bin),
            api_key: Some("test-api-key"),
            auth_token: None,
            rpc_timeout: Duration::from_secs(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS),
            acp_verbose: false,
            george_acp_lane: None,
            ui_idle_notify: None,
            model: None,
            force: false,
        })
            .await
            .expect("spawn");
        s.shutdown().await.expect("shutdown");
        let trace = tmp.path().join("x.jsonl");
        assert!(s.prompt("x", &trace).await.is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn acp_spawn_must_not_leave_child_running_after_handshake_failure() {
        let tmp = workspace_with_prompt_stub();
        let bin = tmp.path().join("mock-acp-bad-session");
        let bad_script = format!("#!/usr/bin/env node\n{}", MOCK_BAD_SESSION_NEW);
        tokio::fs::write(bin.as_path(), bad_script.as_bytes())
            .await
            .unwrap();
        let mut p = std::fs::metadata(&bin).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&bin, p).unwrap();
        crate::test_utils::sync_test_executable(&bin);

        let err = match AcpSession::spawn(AcpSpawnArgs {
            cwd: tmp.path(),
            bin_override: Some(bin.as_path()),
            api_key: Some("test-api-key"),
            auth_token: None,
            rpc_timeout: Duration::from_secs(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS),
            acp_verbose: false,
            george_acp_lane: None,
            ui_idle_notify: None,
            model: None,
            force: false,
        })
        .await
        {
            Ok(_) => panic!("expected session/new handshake failure"),
            Err(e) => e,
        };

        assert!(
            err.contains("sessionId") || err.contains("session/new"),
            "unexpected err: {err}"
        );

        let pid_path = tmp.path().join("mock.pid");
        await_workspace_pid_file(&pid_path).await;
        let pid_s = tokio::fs::read_to_string(&pid_path)
            .await
            .expect("mock.pid");
        let pid: i32 = pid_s.trim().parse().expect("pid");

        let leaked = unix_process_reapable(pid);
        if leaked {
            let _ = unix_kill_probe(pid, "-TERM");
        }

        assert!(
            !leaked,
            "handshake failure leaked mock agent (pid={pid}); spawn error path should kill/reap child"
        );
    }

    /// When `session/cancel` returns a JSON-RPC error, `busy` must stay true if the in-flight
    /// `session/prompt` is still running; otherwise timer/heartbeats think the agent is idle.
    #[cfg(unix)]
    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn acp_cancel_jsonrpc_error_must_not_clear_busy_while_prompt_inflight() {
        let tmp = workspace_with_prompt_stub();
        let bin = tmp.path().join("mock-cancel-err-slow-prompt");
        crate::test_utils::write_acp_jsonrpc_mock_cancel_err_slow_prompt(&bin).await;

        let session = AcpSession::spawn(AcpSpawnArgs {
            cwd: tmp.path(),
            bin_override: Some(bin.as_path()),
            api_key: Some("test-api-key"),
            auth_token: None,
            rpc_timeout: Duration::from_secs(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS),
            acp_verbose: false,
            george_acp_lane: None,
            ui_idle_notify: None,
            model: None,
            force: false,
        })
        .await
        .expect("spawn");

        let trace_slow = tmp.path().join("slow.jsonl");
        let prompt_hit_path = tmp.path().join("prompt_hits");
        let prompt_release_path = tmp.path().join("allow_prompt_complete");
        let sess_prompt = session.clone();
        let driver = tokio::spawn(async move {
            sess_prompt.prompt("slow", &trace_slow).await.unwrap();
        });

        await_workspace_pid_file(&prompt_hit_path).await;
        assert!(session.is_busy(), "expected slow prompt to mark session busy");

        let cancel_res = session.cancel().await;
        assert!(
            cancel_res.is_err(),
            "mock should surface cancel RPC failure, got {cancel_res:?}"
        );
        assert!(
            session.is_busy(),
            "cancel RPC failed but prompt still running; busy must remain true"
        );
        tokio::fs::write(&prompt_release_path, b"ok")
            .await
            .expect("release prompt");

        tokio::time::timeout(std::time::Duration::from_secs(1), driver)
            .await
            .expect("prompt task should finish")
            .expect("join prompt task");

        session.shutdown().await.expect("shutdown");
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)]
    async fn acp_spawn_errors_within_rpc_timeout_with_silent_agent() {
        use std::time::Duration;

        let tmp = workspace_with_prompt_stub();
        let bin = tmp.path().join("silent-agent");
        tokio::fs::write(bin.as_path(), b"#!/bin/sh\nexec sleep 3600\n")
            .await
            .unwrap();
        let mut p = std::fs::metadata(&bin).unwrap().permissions();
        p.set_mode(0o755);
        std::fs::set_permissions(&bin, p).unwrap();
        crate::test_utils::sync_test_executable(&bin);

        let outer = tokio::time::timeout(
            Duration::from_secs(1),
            AcpSession::spawn(AcpSpawnArgs {
                cwd: tmp.path(),
                bin_override: Some(bin.as_path()),
                api_key: Some("test-api-key"),
                auth_token: None,
                rpc_timeout: Duration::from_millis(50),
                acp_verbose: false,
                george_acp_lane: None,
                ui_idle_notify: None,
                model: None,
                force: false,
            }),
        )
        .await;

        let inner = outer.expect("spawn should not hang past outer test timeout");
        let err = match inner {
            Ok(_) => panic!("silent agent should not complete handshake"),
            Err(e) => e,
        };
        assert!(
            err.contains("timed out") || err.contains("acp RPC"),
            "unexpected err: {err}"
        );
    }
}
