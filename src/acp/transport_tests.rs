#![allow(unsafe_code)]
#![allow(clippy::pedantic, clippy::nursery)]

use crate::acp::ReaderLoopInput;
use crate::acp::ResponseTx;
use crate::acp::*;
use serde_json::json;
use std::collections::HashMap;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64};
use std::time::Duration;
use tokio::io::AsyncReadExt;
use tokio::process::{ChildStdin, Command};
use tokio::sync::{Mutex, Notify};

fn acp_activity_state() -> (Arc<AtomicU64>, Arc<Notify>) {
    (Arc::new(AtomicU64::new(0)), Arc::new(Notify::new()))
}

/// Parallel tests mutate global `PATH`; use a fixed path (see `reader_tests` / `ops_inline_tests.inc`).
const SLEEP_BIN: &str = "/bin/sleep";
const TRUE_BIN: &str = "/usr/bin/true";

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
        crate::acp::acp_rpc_timeout(),
        std::time::Duration::from_secs(crate::config::DEFAULT_ACP_RPC_TIMEOUT_SECS)
    );
    unsafe {
        std::env::set_var("MALVIN_ACP_RPC_TIMEOUT_SECS", "5");
    }
    assert_eq!(
        crate::acp::acp_rpc_timeout(),
        std::time::Duration::from_secs(5)
    );
    unsafe {
        std::env::set_var("MALVIN_ACP_RPC_TIMEOUT_SECS", "0");
    }
    assert_eq!(
        crate::acp::acp_rpc_timeout(),
        std::time::Duration::from_secs(1)
    );
    unsafe {
        std::env::remove_var("MALVIN_ACP_RPC_TIMEOUT_SECS");
    }
}

#[test]
fn executable_text_busy_matches_error_kind_and_unix_etxtbsy() {
    use std::io::{Error, ErrorKind};

    assert!(crate::acp::executable_text_busy(&Error::new(
        ErrorKind::ExecutableFileBusy,
        "busy"
    )));
    assert!(!crate::acp::executable_text_busy(&Error::new(
        ErrorKind::NotFound,
        "no"
    )));
    #[cfg(unix)]
    assert!(crate::acp::executable_text_busy(&Error::from_raw_os_error(
        26
    )));
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
            args.windows(2)
                .any(|pair| pair[0] == flag && pair[1] == value),
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
    assert_eq!(
        command_env_value(cmd, "CURSOR_API_KEY").as_deref(),
        expected_key
    );
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
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some("key-a"),
        auth_token: Some("tok-b"),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, Some("key-a"), Some("tok-b"));
}

#[test]
fn test_cursor_credentials_key_only() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some("k-only"),
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
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
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
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
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
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
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some("explicit-wins"),
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
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
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: Some("explicit-tok"),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, None, Some("explicit-tok"));
    clear_cursor_env_for_test();
}

/// Neither explicit nor `CURSOR_*` env: no credentials forwarded.
#[test]
fn test_cursor_credentials_absent_process_env_and_no_explicit() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
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
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some(""),
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
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
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
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
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: Some(""),
        george_acp_lane: None,
        model: None,
        force: false,
    });
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
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, None, None);
    clear_cursor_env_for_test();
}

#[test]
fn test_cursor_credentials_token_only() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: None,
        auth_token: Some("t-only"),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, None, Some("t-only"));
}

#[test]
fn test_cursor_credentials_empty_strings_skipped() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some(""),
        auth_token: Some(""),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, None, None);
}

#[test]
fn test_cursor_credentials_skips_empty_key_only() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some(""),
        auth_token: Some("tok2"),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, None, Some("tok2"));
}

#[test]
fn test_cursor_credentials_skips_empty_token_only() {
    let _guard = crate::test_utils::test_env_lock();
    clear_cursor_env_for_test();
    let tmp = tempfile::tempdir().unwrap();
    let cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(Path::new("/bin/true")),
        api_key: Some("k2"),
        auth_token: Some(""),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    assert_cursor_credentials_forwarding(&cmd, Some("k2"), None);
}

#[tokio::test]
async fn test_write_rpc_line_fails_after_child_stdin_closed() {
    let mut child = Command::new(SLEEP_BIN)
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
    .replace("result: { sessionId: 't1' }", "result: { wrongKey: 't1' }");
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
    acp_activity_seq: Arc<AtomicU64>,
    acp_activity_notify: Arc<Notify>,
    reader_dead: Arc<AtomicBool>,
) {
    let trace_writer: Arc<Mutex<Option<tokio::fs::File>>> = Arc::new(Mutex::new(None));
    tokio::spawn(async move {
        crate::acp::reader_loop(ReaderLoopInput {
            stdout,
            pending,
            stdin,
            acp_activity_seq,
            acp_activity_notify,
            reader_dead,
            trace_writer,
            prompt_cleanup: None,
            acp_verbose: false,
            tee_trace_stdout: false,
        })
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

    let mut cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(&bin),
        api_key: Some(""),
        auth_token: Some(""),
        george_acp_lane: None,
        model: None,
        force: false,
    });
    let mut child = crate::acp::spawn_agent_acp_child(&mut cmd)
        .await
        .expect("spawn");
    let stdin = Arc::new(Mutex::new(child.stdin.take().unwrap()));
    let stdout = child.stdout.take().unwrap();

    let pending = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let reader_dead = Arc::new(AtomicBool::new(false));
    let next_id = Arc::new(AtomicU64::new(1));

    spawn_test_reader_loop(
        stdout,
        pending.clone(),
        stdin.clone(),
        acp_activity_seq.clone(),
        acp_activity_notify.clone(),
        reader_dead.clone(),
    );

    let io = AcpStdioRpc {
        reader_dead,
        stdin,
        pending,
        acp_activity_seq,
        acp_activity_notify,
        acp_verbose: false,
        child_pid: 0,
    };
    let err = handshake_inner(HandshakeParams {
        io: &io,
        next_id: &next_id,
        cwd: tmp.path(),
        rpc_timeout: acp_rpc_timeout(),
        require_cursor_login_auth: true,
    })
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

    let mut cmd = build_agent_acp_command(&BuildAgentAcpCommandArgs {
        cwd: tmp.path(),
        bin_override: Some(&bin),
        api_key: None,
        auth_token: None,
        george_acp_lane: None,
        model: None,
        force: false,
    });
    let mut child = crate::acp::spawn_agent_acp_child(&mut cmd)
        .await
        .expect("spawn");
    let stdin = Arc::new(Mutex::new(child.stdin.take().unwrap()));
    let stdout = child.stdout.take().unwrap();

    let pending = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let reader_dead = Arc::new(AtomicBool::new(false));
    let next_id = Arc::new(AtomicU64::new(1));

    spawn_test_reader_loop(
        stdout,
        pending.clone(),
        stdin.clone(),
        acp_activity_seq.clone(),
        acp_activity_notify.clone(),
        reader_dead.clone(),
    );

    let io = AcpStdioRpc {
        reader_dead,
        stdin,
        pending,
        acp_activity_seq,
        acp_activity_notify,
        acp_verbose: false,
        child_pid: 0,
    };
    let sid = handshake_inner(HandshakeParams {
        io: &io,
        next_id: &next_id,
        cwd: tmp.path(),
        rpc_timeout: acp_rpc_timeout(),
        require_cursor_login_auth: false,
    })
    .await
    .expect("session/new should work without cursor_login authenticate");
    assert_eq!(sid, "t1");

    clear_cursor_env_for_test();
    let _ = child.kill().await;
    let _ = child.wait().await;
}

#[tokio::test]
async fn test_rpc_cancel_when_pending_sender_dropped() {
    let mut child = Command::new(SLEEP_BIN)
        .arg("60")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sleep");
    let stdin = Arc::new(Mutex::new(child.stdin.take().unwrap()));
    let mut stdout = child.stdout.take().unwrap();
    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let reader_dead = Arc::new(AtomicBool::new(false));
    let next_id = Arc::new(AtomicU64::new(1));

    let io = AcpStdioRpc {
        reader_dead,
        stdin: stdin.clone(),
        pending: pending.clone(),
        acp_activity_seq,
        acp_activity_notify,
        acp_verbose: false,
        child_pid: 0,
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
        let r = rpc_request(RpcRequestNext {
            io: &io,
            next_id: &next_id,
            method: "nope",
            params: json!({}),
            rpc_timeout: acp_rpc_timeout(),
        })
        .await;
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
    let mut child = Command::new(TRUE_BIN)
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
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let reader_dead = Arc::new(AtomicBool::new(false));
    let next_id = Arc::new(AtomicU64::new(1));

    let io = AcpStdioRpc {
        reader_dead,
        stdin,
        pending,
        acp_activity_seq,
        acp_activity_notify,
        acp_verbose: false,
        child_pid: 0,
    };
    let err = rpc_request(RpcRequestNext {
        io: &io,
        next_id: &next_id,
        method: "nope",
        params: json!({}),
        rpc_timeout: acp_rpc_timeout(),
    })
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

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[tokio::test]
async fn rpc_timeout_dead_child_reports_exit_not_hung() {
    let mut child = Command::new(SLEEP_BIN)
        .arg("60")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sleep");
    let pid = child.id().expect("child pid");
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
    let mut stdout = child.stdout.take().expect("stdout");
    tokio::spawn(async move {
        let mut buf = [0u8; 256];
        while stdout.read(&mut buf).await.unwrap_or(0) > 0 {}
    });
    let reaper = tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let _ = child.kill().await;
        let _ = child.wait().await;
    });

    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let reader_dead = Arc::new(AtomicBool::new(false));
    let io = AcpStdioRpc {
        reader_dead,
        stdin,
        pending,
        acp_activity_seq,
        acp_activity_notify,
        acp_verbose: false,
        child_pid: pid,
    };
    let err = rpc_request_with_correlation_id(RpcOutgoing {
        io: &io,
        id: 3,
        method: "unanswered",
        params: json!({}),
        rpc_timeout: std::time::Duration::from_millis(100),
    })
    .await
    .expect_err("dead child should fail after silence check");
    let _ = reaper.await;
    assert!(
        err.contains("not running") || err.contains("zombie"),
        "unexpected err: {err}"
    );
}

/// Regression: an inbound JSON-RPC response must win when it arrives during the post-timeout
/// child-health grace sleep — `rpc_wait_response` races `rx` against `evaluate_after_acp_silence`.
#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn rpc_response_arriving_during_child_health_grace_body() {
    let mut child = Command::new(SLEEP_BIN)
        .arg("120")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sleep");
    let pid = child.id().expect("pid");
    tokio::spawn(async move {
        let mut stdout = child.stdout.take().expect("stdout");
        let mut buf = [0u8; 64];
        while stdout.read(&mut buf).await.unwrap_or(0) > 0 {}
        let _ = child.wait().await;
    });

    let (tx, rx) = tokio::sync::oneshot::channel();
    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    tokio::spawn(async move {
        // Fire after the short RPC timeout (~5ms) but inside the minimum ~50ms health grace.
        tokio::time::sleep(Duration::from_millis(15)).await;
        let _ = tx.send(Ok(json!({"delivered": true})));
    });

    let res = rpc_wait_response(RpcWaitArgs {
        pending: &pending,
        acp_activity_seq: &acp_activity_seq,
        acp_activity_notify: &acp_activity_notify,
        id: 42,
        rpc_timeout: Duration::from_millis(5),
        child_pid: pid,
        rx,
    })
    .await
    .expect("response should win over AppearsHung during grace");

    assert_eq!(res["delivered"], true);
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[tokio::test]
async fn rpc_response_arriving_during_child_health_grace_is_delivered() {
    rpc_response_arriving_during_child_health_grace_body().await;
}

#[tokio::test]
async fn rpc_request_with_correlation_id_times_out_when_stdout_silent() {
    let mut child = Command::new(SLEEP_BIN)
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
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let reader_dead = Arc::new(AtomicBool::new(false));
    let io = AcpStdioRpc {
        reader_dead,
        stdin,
        pending,
        acp_activity_seq,
        acp_activity_notify,
        acp_verbose: false,
        child_pid: 0,
    };
    let err = rpc_request_with_correlation_id(RpcOutgoing {
        io: &io,
        id: 3,
        method: "unanswered",
        params: json!({}),
        rpc_timeout: std::time::Duration::from_millis(25),
    })
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
async fn rpc_request_with_correlation_id_errors_when_reader_dead() {
    let reader_dead = Arc::new(AtomicBool::new(true));
    let mut child = Command::new(SLEEP_BIN)
        .arg("2")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("sleep");
    let stdin = Arc::new(Mutex::new(child.stdin.take().expect("stdin")));
    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let io = AcpStdioRpc {
        reader_dead,
        stdin,
        pending,
        acp_activity_seq,
        acp_activity_notify,
        acp_verbose: false,
        child_pid: 0,
    };
    let err = rpc_request_with_correlation_id(RpcOutgoing {
        io: &io,
        id: 7,
        method: "nope",
        params: json!({}),
        rpc_timeout: std::time::Duration::from_millis(500),
    })
    .await
    .expect_err("reader flagged dead");
    assert!(err.contains("dead"), "{err}");
    let _ = child.kill().await;
    let _ = child.wait().await;
}

#[tokio::test]
async fn rpc_request_with_correlation_id_stays_alive_while_json_updates_arrive() {
    let pending: Arc<Mutex<HashMap<u64, ResponseTx>>> = Arc::new(Mutex::new(HashMap::new()));
    let (acp_activity_seq, acp_activity_notify) = acp_activity_state();
    let (tx, rx) = tokio::sync::oneshot::channel();
    let seq = acp_activity_seq.clone();
    let notify = acp_activity_notify.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        note_acp_json_activity(&seq, &notify);
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        note_acp_json_activity(&seq, &notify);
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        let _ = tx.send(Ok(json!({"ok": true})));
    });
    let res = rpc_wait_response(RpcWaitArgs {
        pending: &pending,
        acp_activity_seq: &acp_activity_seq,
        acp_activity_notify: &acp_activity_notify,
        id: 3,
        rpc_timeout: std::time::Duration::from_millis(40),
        child_pid: 0,
        rx,
    })
    .await
    .expect("ACP activity should extend the timeout window");
    assert_eq!(res["ok"], true);
}
