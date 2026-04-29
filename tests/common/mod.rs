#![allow(dead_code)]

#[cfg(unix)]
use std::io::Read;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::{Command, Stdio};
#[cfg(unix)]
use std::thread;
#[cfg(unix)]
use std::time::{Duration, Instant};

#[cfg(unix)]
pub const MALVIN_TEST_CMD_TIMEOUT: Duration = Duration::from_secs(12);

#[cfg(unix)]
fn kill_bin() -> &'static Path {
    if Path::new("/bin/kill").is_file() {
        Path::new("/bin/kill")
    } else {
        Path::new("/usr/bin/kill")
    }
}

#[cfg(unix)]
fn kill_process_group(pid: u32) {
    let _ = Command::new(kill_bin())
        .args(["-KILL", &format!("-{pid}")])
        .status();
}

#[cfg(unix)]
pub fn command_output_with_timeout(
    cmd: &mut Command,
    timeout: Duration,
) -> std::io::Result<std::process::Output> {
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    cmd.process_group(0);
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
                    kill_process_group(child.id());
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

pub fn acp_mock_code_streaming_rich_markdown_js() -> String {
    let heading = session_update_chunk_line("agent_message_chunk", r"'# md-heading-xyz\n'");
    let list = session_update_chunk_line("agent_message_chunk", r"'- md-item-xyz\n'");
    let bold = session_update_chunk_line("agent_message_chunk", r"'**md-bold-xyz**\n'");
    acp_mock_js("", &format!("{heading}\n{list}\n{bold}"))
}

pub fn acp_mock_code_streaming_long_bold_markdown_js() -> String {
    let prompt = format!(
        "    const words = Array(12).fill('wrap-bold-xyz').join(' ');\n{}",
        session_update_chunk_line("agent_message_chunk", r"'**' + words + '**\n'")
    );
    acp_mock_js("", &prompt)
}

pub fn acp_mock_code_abort_after_implement_js() -> String {
    let prompt = r"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    if (promptText.includes('Implement the plan in')) {
      const runRoot = path.join(process.cwd(), '_malvin');
      const runDirNames = fs.readdirSync(runRoot, { withFileTypes: true }).filter((e) => e.isDirectory()).map((e) => e.name).sort();
      fs.writeFileSync(path.join(runRoot, runDirNames[0], 'result.md'), 'ABORT: stop now\n', 'utf8');
      console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: 'implementing\n' } } } }));
    } else {
      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\n', 'utf8');
      console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: 'reviewed\n' } } } }));
    }";
    acp_mock_js("", prompt)
}

fn acp_mock_code_with_run_dir_js(body: &str) -> String {
    let prompt = format!(
        r"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {{}}).prompt || [])[0] || {{}}).text || '';
    const runRoot = path.join(process.cwd(), '_malvin');
    const runDirNames = fs.readdirSync(runRoot, {{ withFileTypes: true }}).filter((e) => e.isDirectory()).map((e) => e.name).sort();
    const runDir = path.join(runRoot, runDirNames[0]);
{body}"
    );
    acp_mock_js("", &prompt)
}

fn chunk_line(text: &str) -> String {
    format!(
        r"      console.log(JSON.stringify({{ jsonrpc: '2.0', method: 'session/update', params: {{ update: {{ sessionUpdate: 'agent_message_chunk', content: {{ type: 'text', text: '{text}\n' }} }} }} }}));"
    )
}

fn write_artifact_lgtm() -> String {
    "      fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\\n', 'utf8');".to_string()
}

fn write_workspace_lgtm() -> String {
    "      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\\n', 'utf8');".to_string()
}

pub fn acp_mock_code_abort_result_after_check_plan_lgtm_js() -> String {
    let lgtm = write_artifact_lgtm();
    let body = format!(
        r"    if (promptText.includes('write ONLY the four characters')) {{
{lgtm}
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: after check plan\n', 'utf8');
{check_done}
    }} else if (promptText.includes('Implement the plan in')) {{
{implement}
    }} else {{
{lgtm}
{reviewed}
    }}",
        check_done = chunk_line("check_plan_done"),
        implement = chunk_line("implement_phase_ran"),
        reviewed = chunk_line("reviewed"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_check_plan_tampers_grounding_then_implement_verifies_restore_js() -> String {
    let lgtm = write_artifact_lgtm();
    let body = format!(
        r#"    if (promptText.includes('write ONLY the four characters "LGTM"')) {{
      fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'TAMPERED\n', 'utf8');
{lgtm}
{checked}
    }} else if (promptText.includes('Implement the plan in')) {{
      const grounding = fs.readFileSync(path.join(process.cwd(), 'grounding.md'), 'utf8');
      if (grounding === 'x') {{
{implement_ok}
      }} else {{
        fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: grounding leaked into implement\n', 'utf8');
{implement_tampered}
      }}
    }} else {{
{lgtm}
{reviewed}
    }}"#,
        checked = chunk_line("checked"),
        implement_ok = chunk_line("implement ok"),
        implement_tampered = chunk_line("implement saw tampered grounding"),
        reviewed = chunk_line("reviewed"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_review_lgtm_to_artifact_js() -> String {
    let lgtm = write_artifact_lgtm();
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
{implement}
    }} else {{
{lgtm}
{reviewed}
    }}",
        implement = chunk_line("implemented"),
        reviewed = chunk_line("lgtm"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_review_lgtm_with_abort_js() -> String {
    let lgtm = write_artifact_lgtm();
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\\n', 'utf8');
    }} else if (promptText.includes('Please review the codebase.')) {{
{lgtm}
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: review lgtm abort test\\n', 'utf8');
    }} else {{
      // no-op for unexpected prompt shapes
    }}",
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_review_writes_workspace_lgtm_js() -> String {
    let body = format!(
        r"    if (promptText.includes('Find a discrepancy between the codebase and')) {{
      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\\n', 'utf8');
    }} else if (promptText.includes('Please review the codebase.')) {{
{workspace_lgtm}
    }} else if (promptText.includes('Concerns')) {{
    }} else {{
      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\\n', 'utf8');
    }}",
        workspace_lgtm = write_workspace_lgtm(),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_check_sync_then_review_lgtm_js() -> String {
    let lgtm = write_artifact_lgtm();
    let body = format!(
        r"    if (promptText.includes('Find a discrepancy between the codebase and')) {{
      let attempts = (typeof this.syncAttempts === 'undefined') ? 0 : this.syncAttempts;
      this.syncAttempts = attempts + 1;
      if (this.syncAttempts === 1) {{
        fs.writeFileSync(path.join(runDir, 'review.md'), 'needs attention\\n', 'utf8');
      }} else {{
{lgtm}
      }}
    }} else if (promptText.includes('Please review the codebase.')) {{
{lgtm}
      {reviewed}
    }} else if (promptText.includes('Concerns')) {{
    }}
",
        lgtm = lgtm,
        reviewed = chunk_line("reviewed"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_sync_review_lgtm_with_abort_js() -> String {
    let lgtm = write_artifact_lgtm();
    let body = format!(
        r"    if (promptText.includes('Find a discrepancy between the codebase and')) {{
      let attempts = (typeof this.syncAttempts === 'undefined') ? 0 : this.syncAttempts;
      this.syncAttempts = attempts + 1;
      if (this.syncAttempts === 1) {{
        fs.writeFileSync(path.join(runDir, 'review.md'), 'needs attention\\n', 'utf8');
      }} else {{
{lgtm}
      }}
    }} else if (promptText.includes('Please review the codebase.')) {{
{lgtm}
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: sync review LGTM abort test\\n', 'utf8');
    }} else if (promptText.includes('Concerns')) {{
    }}",
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_workspace_review_only_lgtm_js() -> String {
    let body = r"    if (!promptText.includes('Concerns')) {{
      const workspaceReview = path.join(process.cwd(), 'review.md');
      const runRootReview = path.join(runRoot, '..', '..', 'review.md');
      fs.writeFileSync(workspaceReview, 'LGTM\\n', 'utf8');
      fs.writeFileSync(runRootReview, 'LGTM\\n', 'utf8');
    }"
    .to_string();
    acp_mock_code_with_run_dir_js(&body)
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

pub fn acp_mock_do_tamper_grounding_and_kissconfig_js() -> String {
    let tamper = r"    const fs = require('fs');
    const path = require('path');
    fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'TAMPERED', 'utf8');
    fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'TAMPERED', 'utf8');";
    let msg = session_update_chunk_line("agent_message_chunk", r"'ok\n'");
    acp_mock_js("", &format!("{tamper}\n{msg}"))
}

pub fn acp_mock_do_creates_grounding_and_kissconfig_js() -> String {
    let create = r"    const fs = require('fs');
    const path = require('path');
    fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'CREATED', 'utf8');
    fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'CREATED', 'utf8');";
    let msg = session_update_chunk_line("agent_message_chunk", r"'ok\n'");
    acp_mock_js("", &format!("{create}\n{msg}"))
}

pub fn acp_mock_sync_tamper_and_review_restore_js() -> String {
    let body = r"    if (promptText.includes('Find a discrepancy between the codebase and')) {
      const syncAttempts = (typeof this.syncAttempts === 'undefined') ? 0 : this.syncAttempts;
      this.syncAttempts = syncAttempts + 1;
      if (syncAttempts === 0) {
        fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'TAMPERED', 'utf8');
        fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'TAMPERED', 'utf8');
      }
      fs.writeFileSync(path.join(runDir, 'review.md'), 'needs attention\n', 'utf8');
    } else if (promptText.includes('Please review the codebase.')) {
      const grounding = (() => { try { return fs.readFileSync(path.join(process.cwd(), 'grounding.md'), 'utf8'); } catch { return ''; } })();
      const kiss = (() => { try { return fs.readFileSync(path.join(process.cwd(), '.kissconfig'), 'utf8'); } catch { return ''; } })();
      if (grounding === 'x' && kiss === 'k') {
        fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
      } else {
        fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: review saw tampered files\n', 'utf8');
      }
    }";
    acp_mock_code_with_run_dir_js(&format!("    {body}"))
}

pub fn acp_mock_sync_reviewer_restore_between_attempts_js() -> String {
    let body = r"    if (promptText.includes('Find a discrepancy between the codebase and')) {
      fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'x', 'utf8');
      fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'k', 'utf8');
    } else if (promptText.includes('Please review the codebase.')) {
      const reviewAttempts = (typeof this.reviewAttempts === 'undefined') ? 0 : this.reviewAttempts;
      this.reviewAttempts = reviewAttempts + 1;
      if (reviewAttempts === 0) {
        fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'TAMPERED', 'utf8');
        fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'needs attention\n', 'utf8');
      } else {
        const grounding = (() => { try { return fs.readFileSync(path.join(process.cwd(), 'grounding.md'), 'utf8'); } catch { return ''; } })();
        const kiss = (() => { try { return fs.readFileSync(path.join(process.cwd(), '.kissconfig'), 'utf8'); } catch { return ''; } })();
        if (grounding === 'x' && kiss === 'k') {
          fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
        } else {
          fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: review still tampered\n', 'utf8');
        }
      }
    }";
    acp_mock_code_with_run_dir_js(&format!("    {body}"))
}

pub fn acp_mock_kpop_tamper_then_restore_js() -> String {
    let body = r"    const fs = require('fs');
    const path = require('path');
    const kpopAttempts = (typeof this.kpopAttempts === 'undefined') ? 0 : this.kpopAttempts;
    this.kpopAttempts = kpopAttempts + 1;
    const grounding = (() => { try { return fs.readFileSync(path.join(process.cwd(), 'grounding.md'), 'utf8'); } catch { return ''; } })();
    const kiss = (() => { try { return fs.readFileSync(path.join(process.cwd(), '.kissconfig'), 'utf8'); } catch { return ''; } })();
    if (kpopAttempts === 0) {
      fs.writeFileSync(path.join(process.cwd(), 'grounding.md'), 'TAMPERED', 'utf8');
      fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'TAMPERED', 'utf8');
    } else if (grounding !== 'x' || kiss !== 'k') {
      fs.writeFileSync(path.join(process.cwd(), 'result.md'), 'ABORT: kpop tamper restored incorrectly\n', 'utf8');
    }";
    let done = session_update_chunk_line("agent_message_chunk", r"'kpop prompt done\n'");
    acp_mock_js("", &format!("    {body}\n{done}"))
}
