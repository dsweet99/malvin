#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::process::Command;

#[cfg(unix)]
const DO_STREAMING_MOCK: &str = r"const fs = require('fs');
const capturePath = process.env.MALVIN_CAPTURE_ARGS_PATH;
if (capturePath) {
  fs.writeFileSync(capturePath, process.argv.slice(2).join('\n'));
}
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
fn run_do_with_mock_and_argv(extra_args: &[&str]) -> (std::process::Output, Vec<String>) {
    let root = tempfile::tempdir().expect("tempdir");
    let home = root.path().join("home");
    let workspace = root.path().join("workspace");
    let capture = root.path().join("captured-argv.txt");
    std::fs::create_dir_all(&home).expect("mkdir home");
    std::fs::create_dir_all(&workspace).expect("mkdir workspace");
    std::fs::write(workspace.join("grounding.md"), "x").expect("grounding");
    let mock = root.path().join("mock-agent-acp-do");
    write_mock_executable(&mock);
    let mut args = vec!["do"];
    args.extend_from_slice(extra_args);
    args.push("say hi");
    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .env("MALVIN_CAPTURE_ARGS_PATH", &capture)
        .args(args)
        .output()
        .expect("spawn malvin do");
    let captured_args = std::fs::read_to_string(&capture)
        .unwrap_or_default()
        .lines()
        .map(std::string::ToString::to_string)
        .collect();
    (out, captured_args)
}

#[cfg(unix)]
fn stdout_lines_preserve_shape(stdout: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(stdout)
        .split('\n')
        .map(|line| line.trim_end_matches('\r').to_string())
        .collect()
}

#[cfg(unix)]
#[test]
fn do_stdout_shows_plain_output_without_jsonrpc_lines() {
    let out = run_do_with_mock(&[]);
    assert!(out.status.success(), "malvin do failed: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout_lines_preserve_shape(&out.stdout),
        vec!["agent message", ""]
    );
    assert!(!stdout.contains("hidden thought"), "stdout was {stdout:?}");
    assert!(!stdout.contains("\"jsonrpc\""), "stdout was {stdout:?}");
}

#[cfg(unix)]
#[test]
fn do_stdout_includes_thoughts_only_with_flag() {
    let out = run_do_with_mock(&["--thoughts"]);
    assert!(out.status.success(), "malvin do --thoughts failed: {out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines = stdout_lines_preserve_shape(&out.stdout);
    assert_eq!(lines.len(), 3, "unexpected stdout shape: {lines:?}");
    assert_eq!(lines[0], "agent message");
    assert!(lines[1].contains("hidden thought"), "stdout was {stdout:?}");
    assert_eq!(lines[2], "");
    assert!(stdout.contains("hidden thought"), "stdout was {stdout:?}");
    assert!(!stdout.contains("\"jsonrpc\""), "stdout was {stdout:?}");
}

#[cfg(unix)]
#[test]
fn do_forwards_default_model_and_force_to_agent() {
    let (out, argv) = run_do_with_mock_and_argv(&[]);
    assert!(out.status.success(), "malvin do failed: {out:?}");
    let model_values: Vec<&str> = argv
        .windows(2)
        .filter(|w| w[0] == "--model")
        .map(|w| w[1].as_str())
        .collect();
    assert!(
        model_values == vec!["composer-2"],
        "expected exactly one forwarded --model composer-2; argv={argv:?}"
    );
    let force_count = argv.iter().filter(|arg| arg.as_str() == "--force").count();
    assert!(
        force_count == 1,
        "expected exactly one forwarded --force by default; argv={argv:?}"
    );
    assert!(
        !argv.iter().any(|arg| arg == "--no-force"),
        "did not expect forwarded --no-force; argv={argv:?}"
    );
}

#[cfg(unix)]
#[test]
fn do_respects_no_force_and_explicit_model_flags() {
    let root = tempfile::tempdir().expect("tempdir");
    let home = root.path().join("home");
    let workspace = root.path().join("workspace");
    let capture = root.path().join("captured-argv.txt");
    std::fs::create_dir_all(&home).expect("mkdir home");
    std::fs::create_dir_all(&workspace).expect("mkdir workspace");
    std::fs::write(workspace.join("grounding.md"), "x").expect("grounding");
    let mock = root.path().join("mock-agent-acp-do");
    write_mock_executable(&mock);
    let out = Command::new(env!("CARGO_BIN_EXE_malvin"))
        .current_dir(&workspace)
        .env("HOME", &home)
        .env("CURSOR_AGENT_API_KEY", "test-key")
        .env("MALVIN_AGENT_ACP_BIN", &mock)
        .env("MALVIN_CAPTURE_ARGS_PATH", &capture)
        .args(["--no-force", "--model", "composer-x", "do", "say hi"])
        .output()
        .expect("spawn malvin do");
    assert!(out.status.success(), "malvin do failed: {out:?}");
    let argv: Vec<String> = std::fs::read_to_string(&capture)
        .unwrap_or_default()
        .lines()
        .map(std::string::ToString::to_string)
        .collect();
    let model_values: Vec<&str> = argv
        .windows(2)
        .filter(|w| w[0] == "--model")
        .map(|w| w[1].as_str())
        .collect();
    assert!(
        model_values == vec!["composer-x"],
        "expected exactly one forwarded --model composer-x; argv={argv:?}"
    );
    assert!(
        !argv.iter().any(|arg| arg == "--force"),
        "did not expect --force with --no-force; argv={argv:?}"
    );
    assert!(
        !argv.iter().any(|arg| arg == "--no-force"),
        "did not expect forwarded --no-force; argv={argv:?}"
    );
}
