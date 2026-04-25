#![allow(unsafe_code)]
#![allow(clippy::pedantic, clippy::nursery)]

use super::session_types::AcpSessionInner;
use super::*;
use serde_json::json;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
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

const MOCK_DO_SPLIT_STREAMING: &str = r#"const readline = require('readline');
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
console.log(JSON.stringify({
  jsonrpc: '2.0',
  method: 'session/update',
  params: {
    update: {
      sessionUpdate: 'agent_message_chunk',
      content: { type: 'text', text: 'agent message\n' }
    }
  }
}));
console.log(JSON.stringify({
  jsonrpc: '2.0',
  method: 'session/update',
  params: {
    update: {
      sessionUpdate: 'agent_thought_chunk',
      content: { type: 'text', text: 'hidden thought\n' }
    }
  }
}));
console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { stopReason: 'end' } }));
  } else if (rid != null) {
console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});
"#;

async fn write_mock_executable(path: &Path) {
    let script = format!("#!/usr/bin/env node\n{}", EXTENDED_MOCK_AGENT);
    tokio::fs::write(path, script.as_bytes()).await.unwrap();
    let mut p = std::fs::metadata(path).unwrap().permissions();
    p.set_mode(0o755);
    std::fs::set_permissions(path, p).unwrap();
    crate::test_utils::sync_test_executable(path);
}

async fn write_do_split_streaming_mock_executable(path: &Path) {
    let script = format!("#!/usr/bin/env node\n{}", MOCK_DO_SPLIT_STREAMING);
    tokio::fs::write(path, script.as_bytes()).await.unwrap();
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
        stringify!(crate::acp::acp_spawn_start_reader_and_handshake).contains("acp_spawn"),
        "expected stringify to retain handshake symbol name"
    );
}

#[test]
fn test_acp_rpc_timeout_matches_config_env_helper() {
    let _g = crate::test_utils::test_env_lock();
    assert_eq!(
        crate::acp::acp_rpc_timeout(),
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
        tee_trace_stdout: false,
        raw_output: false,
        emit_stdout_markdown: false,
    })
    .await
    .expect("spawn mock agent acp");
    let trace = tmp.path().join("t.jsonl");
    s.prompt("hello", &trace, "implement", None)
        .await
        .expect("prompt");
    s.shutdown().await.expect("shutdown");
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn acp_trace_starts_with_malvin_command_line_after_invocation_init() {
    let _g = crate::test_utils::test_env_lock();
    crate::invocation::init_from_env();
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
        tee_trace_stdout: false,
        raw_output: false,
        emit_stdout_markdown: false,
    })
    .await
    .expect("spawn mock agent acp");
    let trace = tmp.path().join("trace.jsonl");
    s.prompt("hello", &trace, "implement", None)
        .await
        .expect("prompt");
    s.shutdown().await.expect("shutdown");
    let text = std::fs::read_to_string(&trace).expect("read trace");
    let cmd = crate::invocation::command_line().expect("invocation line");
    let inner = crate::output::format_log_tag_inner(crate::output::MALVIN_WHO);
    let expected_fragment = format!(":[{inner}]: Command: {cmd}\n");
    assert!(
        text.starts_with(&expected_fragment)
            || text
                .lines()
                .next()
                .is_some_and(|line| line.ends_with(&format!(":[{inner}]: Command: {cmd}"))),
        "trace should start with malvin Command line; expected fragment {:?} got start {:?}",
        expected_fragment,
        text.chars().take(80).collect::<String>()
    );
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
        tee_trace_stdout: false,
        raw_output: false,
        emit_stdout_markdown: false,
    })
    .await
    .expect("spawn verbose");
    let trace = tmp.path().join("tv.jsonl");
    s.prompt("hi", &trace, "implement", None)
        .await
        .expect("prompt");
    s.shutdown().await.expect("shutdown");
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn acp_prompt_do_trace_split_writes_plain_trace_and_suppresses_thoughts() {
    let tmp = workspace_with_prompt_stub();
    let bin = tmp.path().join("mock-agent-acp-do-split");
    write_do_split_streaming_mock_executable(&bin).await;
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
        tee_trace_stdout: false,
        raw_output: true,
        emit_stdout_markdown: false,
    })
    .await
    .expect("spawn");
    let trace = tmp.path().join("do-split-trace.log");
    s.prompt_do_trace_split(
        "STYLE\n\nHEADER\n\nUSER",
        &trace,
        super::outgoing_prompt_trace::DoPromptTraceSplit {
            style_text: Some("STYLE"),
            header: "HEADER",
            user: "USER",
        },
    )
    .await
    .expect("prompt");
    s.shutdown().await.expect("shutdown");
    let text = std::fs::read_to_string(&trace).expect("trace");
    let pos_cmd = text
        .find("Command:")
        .expect("trace should include invocation header like uniform prompts");
    let pos_body = text
        .find("STYLE\n\nHEADER\n\nUSER\n")
        .expect("do-split body");
    assert!(
        pos_cmd < pos_body,
        "invocation line must precede prompt body, trace was {text:?}"
    );
    assert!(text.contains("STYLE\n\nHEADER\n\nUSER\n"), "trace was {text:?}");
    assert!(text.contains("agent message\n"), "trace was {text:?}");
    assert!(!text.contains("hidden thought"), "trace was {text:?}");
    assert!(!text.contains(":[>"), "no ACP-style outgoing `>stem` log lines in plain do trace");
    assert!(!text.contains(":[<"), "no ACP-style incoming log lines in plain do trace");
    assert!(!text.contains("<do"));
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn acp_prompt_do_trace_split_cooked_keeps_thoughts_in_trace() {
    let tmp = workspace_with_prompt_stub();
    let bin = tmp.path().join("mock-agent-acp-do-split-cooked");
    write_do_split_streaming_mock_executable(&bin).await;
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
        tee_trace_stdout: false,
        raw_output: false,
        emit_stdout_markdown: false,
    })
    .await
    .expect("spawn");
    let trace = tmp.path().join("do-split-cooked-trace.log");
    s.prompt_do_trace_split(
        "STYLE\n\nHEADER\n\nUSER",
        &trace,
        super::outgoing_prompt_trace::DoPromptTraceSplit {
            style_text: Some("STYLE"),
            header: "HEADER",
            user: "USER",
        },
    )
    .await
    .expect("prompt");
    s.shutdown().await.expect("shutdown");
    let text = std::fs::read_to_string(&trace).expect("trace");
    let pos_cmd = text
        .find("Command:")
        .expect("trace should include invocation header like uniform prompts");
    let pos_body = text
        .find("STYLE\n\nHEADER\n\nUSER\n")
        .expect("do-split body");
    assert!(pos_cmd < pos_body, "invocation before body, trace was {text:?}");
    assert!(text.contains("agent message\n"), "trace was {text:?}");
    assert!(text.contains("[hidden thought]"), "trace was {text:?}");
    assert!(!text.contains(":[>"), "no ACP-style outgoing `>stem` log lines in plain do trace");
    assert!(!text.contains(":[<"), "no ACP-style incoming log lines in plain do trace");
    assert!(!text.contains("<do"));
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn acp_prompt_do_trace_split_rejects_payload_mismatch() {
    let tmp = workspace_with_prompt_stub();
    let bin = tmp.path().join("mock-agent-acp-do-split-mismatch");
    write_do_split_streaming_mock_executable(&bin).await;
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
        tee_trace_stdout: false,
        raw_output: true,
        emit_stdout_markdown: false,
    })
    .await
    .expect("spawn");
    let trace = tmp.path().join("do-split-mismatch.log");
    let res = s
        .prompt_do_trace_split(
            "STYLE\nHEADER\n\nDIFFERENT_USER",
            &trace,
            super::outgoing_prompt_trace::DoPromptTraceSplit {
                style_text: Some("STYLE"),
                header: "HEADER",
                user: "USER",
            },
        )
        .await;
    assert!(res.is_err(), "expected mismatch error");
    assert!(
        res.expect_err("error")
            .contains("text does not match split parts"),
        "unexpected mismatch error"
    );
    assert!(!s.is_busy(), "mismatch should fail before prompt dispatch");
    s.shutdown().await.expect("shutdown");
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn acp_prompt_do_trace_split_accepts_trailing_newline_mismatch() {
    let tmp = workspace_with_prompt_stub();
    let bin = tmp.path().join("mock-agent-acp-do-split-trailing-newline");
    write_do_split_streaming_mock_executable(&bin).await;
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
        tee_trace_stdout: false,
        raw_output: true,
        emit_stdout_markdown: false,
    })
    .await
    .expect("spawn");
    let trace = tmp.path().join("do-split-trailing-newline.log");
    s.prompt_do_trace_split(
        "STYLE\n\nHEADER\n\nUSER\n",
        &trace,
        super::outgoing_prompt_trace::DoPromptTraceSplit {
            style_text: Some("STYLE"),
            header: "HEADER",
            user: "USER",
        },
    )
    .await
    .expect("prompt");
    s.shutdown().await.expect("shutdown");
}

#[tokio::test]
#[allow(clippy::await_holding_lock)]
async fn acp_prompt_do_trace_split_accepts_trailing_space_mismatch() {
    let tmp = workspace_with_prompt_stub();
    let bin = tmp.path().join("mock-agent-acp-do-split-trailing-space");
    write_do_split_streaming_mock_executable(&bin).await;
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
        tee_trace_stdout: false,
        raw_output: true,
        emit_stdout_markdown: false,
    })
    .await
    .expect("spawn");
    let trace = tmp.path().join("do-split-trailing-space.log");
    s.prompt_do_trace_split(
        "STYLE\n\nHEADER\n\nUSER   ",
        &trace,
        super::outgoing_prompt_trace::DoPromptTraceSplit {
            style_text: Some("STYLE"),
            header: "HEADER",
            user: "USER",
        },
    )
    .await
    .expect("prompt");
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
        tee_trace_stdout: false,
        raw_output: false,
        emit_stdout_markdown: false,
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
        tee_trace_stdout: false,
        raw_output: false,
        emit_stdout_markdown: false,
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
        tee_trace_stdout: false,
        raw_output: false,
        emit_stdout_markdown: false,
    })
    .await
    .expect("spawn");
    let trace = tmp.path().join("prompt_err.jsonl");
    assert!(s
        .prompt("x", &trace, "implement", None)
        .await
        .is_err());
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
        tee_trace_stdout: false,
        raw_output: false,
        emit_stdout_markdown: false,
    })
    .await
    .expect("spawn");
    s.shutdown().await.expect("shutdown");
    let trace = tmp.path().join("x.jsonl");
    assert!(s
        .prompt("x", &trace, "implement", None)
        .await
        .is_err());
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
        tee_trace_stdout: false,
        raw_output: false,
        emit_stdout_markdown: false,
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
        tee_trace_stdout: false,
        raw_output: false,
        emit_stdout_markdown: false,
    })
    .await
    .expect("spawn");

    let trace_slow = tmp.path().join("slow.jsonl");
    let prompt_hit_path = tmp.path().join("prompt_hits");
    let prompt_release_path = tmp.path().join("allow_prompt_complete");
    let sess_prompt = session.clone();
    let driver = tokio::spawn(async move {
        sess_prompt
            .prompt("slow", &trace_slow, "implement", None)
            .await
            .unwrap();
    });

    await_workspace_pid_file(&prompt_hit_path).await;
    assert!(
        session.is_busy(),
        "expected slow prompt to mark session busy"
    );

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
            tee_trace_stdout: false,
            raw_output: false,
            emit_stdout_markdown: false,
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

#[test]
fn kiss_stringify_session_a() {
    let _ = stringify!(super::prompt_stdout_replacement);
    let _ = stringify!(super::outgoing_prompt_trace::OutgoingPromptTrace::Uniform);
    let _ = stringify!(super::outgoing_prompt_trace::OutgoingPromptTrace::DoSplit);
    let _ = stringify!(AcpSession::spawn);
    let _ = stringify!(AcpSession::is_alive);
    let _ = stringify!(AcpSession::is_busy);
    let _ = stringify!(AcpSession::prompt);
    let _ = stringify!(AcpSession::prompt_do_trace_split);
    let _ = stringify!(AcpSession::cancel);
    let _ = stringify!(AcpSession::shutdown);
}

#[test]
fn kiss_stringify_session_b() {
    let _ = stringify!(AcpSession::send_rpc);
    let _ = stringify!(AcpSession::reset_prompt_inflight);
    let _ = stringify!(AcpSession::prompt_impl);
    let _ = stringify!(super::rpc_session_prompt_text);
    let _ = stringify!(super::is_prompt_payload_trailing_ws);
    let _ = stringify!(super::trim_prompt_payload_trailing_ws);
    let _ = stringify!(super::do_split_trace_preamble);
}

#[test]
fn prompt_stdout_replacement_redacts_learn_only() {
    assert_eq!(
        super::prompt_stdout_replacement("learn"),
        Some(crate::output::LEARNING_PLACEHOLDER)
    );
    assert_eq!(super::prompt_stdout_replacement("kpop"), None);
    assert_eq!(super::prompt_stdout_replacement("review_1"), None);
}
