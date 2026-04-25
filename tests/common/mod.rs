#![allow(dead_code)]

#[cfg(unix)]
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::process::{Command, Stdio};
#[cfg(unix)]
use std::thread;
#[cfg(unix)]
use std::time::{Duration, Instant};

#[cfg(unix)]
pub const MALVIN_TEST_CMD_TIMEOUT: Duration = Duration::from_secs(12);

#[cfg(unix)]
pub fn command_output_with_timeout(
    cmd: &mut Command,
    timeout: Duration,
) -> std::io::Result<std::process::Output> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().expect("stdout piped");
    let stderr = child.stderr.take().expect("stderr piped");
    let stdout_jh = thread::spawn(move || {
        let mut stdout = stdout;
        let mut v = Vec::new();
        let _ = stdout.read_to_end(&mut v);
        v
    });
    let stderr_jh = thread::spawn(move || {
        let mut stderr = stderr;
        let mut v = Vec::new();
        let _ = stderr.read_to_end(&mut v);
        v
    });
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = stdout_jh.join().map_err(|_| {
                    std::io::Error::other("malvin subprocess stdout reader panicked")
                })?;
                let stderr = stderr_jh.join().map_err(|_| {
                    std::io::Error::other("malvin subprocess stderr reader panicked")
                })?;
                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    let _ = stdout_jh.join();
                    let _ = stderr_jh.join();
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::TimedOut,
                        "malvin subprocess timed out",
                    ));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => {
                let _ = stdout_jh.join();
                let _ = stderr_jh.join();
                return Err(e);
            }
        }
    }
}

pub fn test_home_workspace() -> (tempfile::TempDir, std::path::PathBuf, std::path::PathBuf) {
    let root = tempfile::tempdir().expect("tempdir");
    let home = root.path().join("home");
    let workspace = root.path().join("workspace");
    std::fs::create_dir_all(&home).expect("mkdir home");
    std::fs::create_dir_all(&workspace).expect("mkdir workspace");
    std::fs::write(workspace.join("grounding.md"), "x").expect("grounding");
    (root, home, workspace)
}

#[cfg(unix)]
pub fn write_fake_kiss(path: &std::path::Path) {
    std::fs::write(path, "#!/usr/bin/env sh\nexit 0\n").expect("write kiss");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
pub fn write_mock_executable(path: &std::path::Path, js: &str) {
    let script = format!("#!/usr/bin/env node\n{js}");
    std::fs::write(path, script).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

const ARGV_CAPTURE_PREAMBLE: &str = r"const fs = require('fs');
const capturePath = process.env.MALVIN_CAPTURE_ARGS_PATH;
if (capturePath) {
  fs.writeFileSync(capturePath, process.argv.slice(2).join('\n'));
}
";

fn acp_mock_js(preamble: &str, prompt_handler: &str) -> String {
    format!(
        r"{preamble}const readline = require('readline');
const rl = readline.createInterface({{ input: process.stdin, crlfDelay: Infinity }});
rl.on('line', (line) => {{
  line = line.trim();
  if (!line) return;
  let msg;
  try {{ msg = JSON.parse(line); }} catch (e) {{ return; }}
  const mid = msg.method;
  const rid = msg.id;
  if (mid === 'initialize') {{
    console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{}} }}));
  }} else if (mid === 'authenticate') {{
    console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{}} }}));
  }} else if (mid === 'session/new') {{
    console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{ sessionId: 't1' }} }}));
  }} else if (mid === 'session/prompt') {{
{prompt_handler}
    console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{ stopReason: 'end' }} }}));
  }} else if (rid != null) {{
    console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{}} }}));
  }}
}});"
    )
}

fn session_update_chunk_line(kind: &str, text_expr: &str) -> String {
    format!(
        r"    console.log(JSON.stringify({{ jsonrpc: '2.0', method: 'session/update', params: {{ update: {{ sessionUpdate: '{kind}', content: {{ type: 'text', text: {text_expr} }} }} }} }}));"
    )
}

pub fn acp_mock_code_streaming_update_js() -> String {
    let prompt = session_update_chunk_line("agent_message_chunk", r"'agent message\n'");
    acp_mock_js("", &prompt)
}

pub fn acp_mock_code_streaming_bold_markdown_js() -> String {
    let prompt = session_update_chunk_line("agent_message_chunk", r"'**boldline**\n'");
    acp_mock_js("", &prompt)
}

pub fn acp_mock_do_streaming_update_js() -> String {
    let msg = session_update_chunk_line("agent_message_chunk", r"'agent message\n'");
    let thought = session_update_chunk_line("agent_thought_chunk", r"'hidden thought\n'");
    acp_mock_js(ARGV_CAPTURE_PREAMBLE, &format!("{msg}\n{thought}"))
}

pub fn acp_mock_do_streaming_wordy_long_msg_js() -> String {
    let prompt = format!(
        "    const words = Array(15).fill('abcdefghij').join(' ');\n{}",
        session_update_chunk_line("agent_message_chunk", r"words + '\n'")
    );
    acp_mock_js("", &prompt)
}

pub fn acp_mock_do_streaming_long_agent_msg_js() -> String {
    let prompt = format!(
        "    const long = 'a'.repeat(120);\n{}",
        session_update_chunk_line("agent_message_chunk", r"long + '\n'")
    );
    acp_mock_js(ARGV_CAPTURE_PREAMBLE, &prompt)
}

pub fn acp_mock_do_tampers_grounding_js() -> String {
    let tamper = r"    const fs = require('fs');
    const path = require('path');
    fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'TAMPERED', 'utf8');";
    let msg = session_update_chunk_line("agent_message_chunk", r"'ok\n'");
    let thought = session_update_chunk_line("agent_thought_chunk", r"'t\n'");
    acp_mock_js("", &format!("{tamper}\n{msg}\n{thought}"))
}
