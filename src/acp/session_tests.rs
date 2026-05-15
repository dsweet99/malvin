#[test]
fn kiss_stringify_acp_session_units() {
    let _ = stringify!(crate::acp::session::prompt_stdout_replacement);
    let _ = stringify!(crate::acp::session::rpc_session_prompt_text);
    let _ = stringify!(crate::acp::session::do_split_trace_preamble);
}

#[cfg(unix)]
fn process_exists(pid: u32) -> bool {
    std::process::Command::new("kill")
        .arg("-0")
        .arg(pid.to_string())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(unix)]
async fn wait_for_pid_file(path: &std::path::Path) -> u32 {
    for _ in 0..100 {
        if let Ok(raw) = tokio::fs::read_to_string(path).await {
            if let Ok(pid) = raw.trim().parse::<u32>() {
                return pid;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    panic!("pid file was not written: {}", path.display());
}

#[cfg(unix)]
async fn write_descendant_spawning_acp_mock(bin: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let script = r"#!/usr/bin/env node
const readline = require('readline');
const { spawnSync } = require('child_process');
const rl = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
function send(id, result) {
  console.log(JSON.stringify({ jsonrpc: '2.0', id, result }));
}
rl.on('line', (line) => {
  line = line.trim();
  if (!line) return;
  const msg = JSON.parse(line);
  if (msg.method === 'initialize') {
    send(msg.id, {});
  } else if (msg.method === 'authenticate') {
    send(msg.id, {});
  } else if (msg.method === 'session/new') {
    send(msg.id, { sessionId: 't1' });
  } else if (msg.method === 'session/prompt') {
    spawnSync('/bin/sh', ['-c', 'nohup sleep 30 >/dev/null 2>&1 & echo $! > descendant.pid'], { cwd: process.cwd() });
    send(msg.id, {});
  } else {
    send(msg.id, {});
  }
});
";
    tokio::fs::write(bin, script.as_bytes()).await.unwrap();
    let mut perms = std::fs::metadata(bin).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(bin, perms).unwrap();
    crate::test_utils::sync_test_executable(bin);
}

#[cfg(unix)]
#[tokio::test]
async fn shutdown_kills_agent_spawned_descendants() {
    let tmp = tempfile::tempdir().unwrap();
    let bin = tmp.path().join("descendant-spawning-agent");
    write_descendant_spawning_acp_mock(&bin).await;

    let session = crate::acp::AcpSession::spawn(super::spawn_test_args::george_mock_spawn_args(
        tmp.path(),
        &bin,
    ))
    .await
    .expect("mock acp session should start");
    session
        .prompt("spawn descendant", &tmp.path().join("prompt.log"), "test", None)
        .await
        .expect("mock prompt should complete");

    let pid = wait_for_pid_file(&tmp.path().join("descendant.pid")).await;
    assert!(process_exists(pid), "descendant should be alive before shutdown");

    session.shutdown().await.expect("shutdown should complete");
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    if process_exists(pid) {
        let _ = std::process::Command::new("kill")
            .arg("-KILL")
            .arg(pid.to_string())
            .status();
        panic!("shutdown left agent-spawned descendant process {pid} alive");
    }
}

#[tokio::test]
#[allow(unsafe_code)]
#[allow(clippy::await_holding_lock)]
async fn acp_session_cancel_clears_busy_state_after_rpc_error() {
    use std::{sync::Arc, time::Duration};
    use std::sync::atomic::Ordering;
    use std::process::Stdio;

    let mut child = tokio::process::Command::new("cat")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn cat");
    let stdin = child.stdin.take().expect("stdin");

    let session = crate::acp::AcpSession(Arc::new(crate::acp::session_types::AcpSessionInner {
        child: tokio::sync::Mutex::new(child),
        process_group_id: None,
        stdin: Arc::new(tokio::sync::Mutex::new(stdin)),
        pending: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::default())),
        acp_activity_seq: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        acp_activity_notify: Arc::new(tokio::sync::Notify::new()),
        next_id: Arc::new(std::sync::atomic::AtomicU64::new(1)),
        session_id: "session-id".to_string(),
        reader_dead: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        rpc_timeout: Duration::from_millis(100),
        busy: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        trace_writer: Arc::new(tokio::sync::Mutex::new(None)),
        prompt_rpc_id: Arc::new(std::sync::atomic::AtomicU64::new(123)),
        prompt_singleflight: Arc::new(tokio::sync::Mutex::new(())),
        acp_verbose: false,
        ui_idle_notify: None,
        raw_output: false,
        show_thoughts_on_stdout: false,
        emit_stdout_markdown: false,
        prompts_log_run_dir: None,
        log_full_outgoing_prompts: false,
    }));

    let err = session.cancel().await.expect_err("cancel should fail on dead transport");
    assert!(err.contains("session is dead"), "{err}");
    assert!(!session.is_busy());
    assert_eq!(session.0.prompt_rpc_id.load(Ordering::SeqCst), 0);
    assert!(session.0.trace_writer.lock().await.is_none());
}
