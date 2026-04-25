#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[cfg(unix)]
const DO_STREAMING_MOCK: &str = r"const readline = require('readline');
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
";

#[cfg(unix)]
fn write_mock_executable(path: &std::path::Path) {
    let script = format!("#!/usr/bin/env node\n{DO_STREAMING_MOCK}");
    std::fs::write(path, script).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
fn run_do_with_mock(extra_args: &[&str]) -> std::process::Output {
    let root = tempfile::tempdir().expect("tempdir");
    let home = root.path().join("home");
    let workspace = root.path().join("workspace");
    std::fs::create_dir_all(&home).expect("mkdir home");
    std::fs::create_dir_all(&workspace).expect("mkdir workspace");
    std::fs::write(workspace.join("grounding.md"), "x").expect("grounding");
    let mock = root.path().join("mock-agent-acp-do");
    write_mock_executable(&mock);
    let mut args = vec!["do"];
    args.extend_from_slice(extra_args);
    args.push("say hi");
    Command::new(env!("CARGO_BIN_EXE_malvin"))
        .current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .args(args)
        .output()
        .expect("spawn malvin do")
}

#[cfg(unix)]
fn stdout_nonempty_lines(stdout: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(stdout)
        .lines()
        .map(|line| line.trim_end_matches('\r').to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

#[cfg(unix)]
#[test]
fn do_stdout_shows_plain_output_without_jsonrpc_lines() {
    let out = run_do_with_mock(&[]);
    assert!(out.status.success(), "malvin do failed: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(stdout_nonempty_lines(&out.stdout), vec!["agent message"]);
    assert!(!stdout.contains("hidden thought"), "stdout was {stdout:?}");
    assert!(!stdout.contains("\"jsonrpc\""), "stdout was {stdout:?}");
    assert!(!stdout.contains("<do"), "stdout was {stdout:?}");
    assert!(!stdout.contains(":["), "stdout was {stdout:?}");
}

#[cfg(unix)]
#[test]
fn do_stdout_includes_thoughts_only_with_flag() {
    let out = run_do_with_mock(&["--thoughts"]);
    assert!(out.status.success(), "malvin do --thoughts failed: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout_nonempty_lines(&out.stdout),
        vec!["agent message", "[hidden thought]"]
    );
    assert!(stdout.contains("hidden thought"), "stdout was {stdout:?}");
    assert!(!stdout.contains("\"jsonrpc\""), "stdout was {stdout:?}");
    assert!(!stdout.contains("<do"), "stdout was {stdout:?}");
    assert!(!stdout.contains(":["), "stdout was {stdout:?}");
}
