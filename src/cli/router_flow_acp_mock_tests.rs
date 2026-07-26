//! Mock ACP agent helpers for router flow tests.

#[cfg(unix)]
pub(crate) fn install_mock_router_agent_env_with_script(
    workspace: &std::path::Path,
    mock: &std::path::Path,
) -> crate::test_utils::SavedEnvVars {
    #![allow(unsafe_code)]

    let bin_dir = workspace.join("bin");
    std::fs::create_dir_all(&bin_dir).expect("mkdir bin");
    crate::test_agent_client::install_exit_gate_bin(&bin_dir, "kiss", 0);
    let guard = crate::test_utils::SavedEnvVars::capture(&[
        "MALVIN_AGENT_ACP_BIN",
        "PATH",
        "CURSOR_AGENT_API_KEY",
        crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV,
    ]);
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    unsafe {
        std::env::set_var("MALVIN_AGENT_ACP_BIN", mock);
        std::env::set_var("PATH", path);
        std::env::set_var("CURSOR_AGENT_API_KEY", "test-key");
        std::env::set_var(crate::acp::MALVIN_TEST_NO_REAL_AGENT_ENV, "1");
    }
    guard
}

#[cfg(unix)]
pub(crate) fn install_mock_router_agent_env(
    workspace: &std::path::Path,
    mock: &std::path::Path,
) -> crate::test_utils::SavedEnvVars {
    write_mock_router_agent(mock);
    install_mock_router_agent_env_with_script(workspace, mock)
}

#[cfg(unix)]
pub(crate) fn write_mock_router_agent_session_fail(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let script = r"#!/usr/bin/env node
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
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, error: { code: -32603, message: 'session fail' } }));
  } else if (rid != null) {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});
";
    std::fs::write(path, script.as_bytes()).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

/// Workspace-relative path where the mock writes `session/new` + prompt counts.
#[cfg(unix)]
pub(crate) const ROUTER_MOCK_SESSION_COUNTS_FILE: &str = ".malvin_router_mock_session_counts.json";

/// Writes one group JSON on the requirements turn, then answers group `KPop` + work turns.
/// Counts `session/new` and `session/prompt` into [`ROUTER_MOCK_SESSION_COUNTS_FILE`] under cwd.
#[cfg(unix)]
pub(crate) fn write_mock_router_agent(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let script = format!(
        r#"#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const countsPath = path.resolve(process.cwd(), '{ROUTER_MOCK_SESSION_COUNTS_FILE}');
const counts = {{ begins: 0, prompts: 0, ends: 0 }};
function flushCounts() {{
  try {{ fs.writeFileSync(countsPath, JSON.stringify(counts)); }} catch (e) {{}}
}}
process.on('exit', () => {{ counts.ends += 1; flushCounts(); }});
const readline = require('readline');
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
    counts.begins += 1;
    flushCounts();
    console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{ sessionId: 't1' }} }}));
  }} else if (mid === 'session/prompt') {{
    counts.prompts += 1;
    flushCounts();
    if (!global.pc) global.pc = 0;
    global.pc++;
    const promptText = (msg.params && msg.params.prompt)
      ? (Array.isArray(msg.params.prompt)
          ? msg.params.prompt.map(p => (p && p.text) || '').join('\n')
          : String(msg.params.prompt))
      : '';
    if (global.pc === 1) {{
      let outPath = null;
      const abs = promptText.match(/(\/[^\s`"']+review_requirements\.json)/);
      const rel = promptText.match(/(\.\/[^\s`"']+review_requirements\.json)/);
      if (abs) outPath = abs[1];
      else if (rel) outPath = rel[1];
      if (outPath) {{
        const resolved = path.isAbsolute(outPath) ? outPath : path.resolve(process.cwd(), outPath);
        fs.mkdirSync(path.dirname(resolved), {{ recursive: true }});
        fs.writeFileSync(resolved, JSON.stringify({{
          groups: [{{ title: 'G1', requirements: ['satisfy mock requirement'] }}]
        }}));
      }}
    }}
    const responses = [
      'router_requirements phase\nwrote review_requirements.json\n',
      'router_kpop_group phase\nresidual plan: do the mock work\n',
      'router_work done\n'
    ];
    const text = responses[Math.min(global.pc - 1, responses.length - 1)];
    console.log(JSON.stringify({{ jsonrpc: '2.0', method: 'session/update', params: {{ update: {{ sessionUpdate: 'agent_message_chunk', content: {{ type: 'text', text }} }} }} }}));
    console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{ stopReason: 'end' }} }}));
  }} else if (rid != null) {{
    console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{}} }}));
  }}
}});
"#
    );
    std::fs::write(path, script.as_bytes()).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

/// Requirements turn that does not write JSON (parse should fail).
#[cfg(unix)]
pub(crate) fn write_mock_router_agent_missing_requirements(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let handler = r"    if (!global.pc) global.pc = 0;
    global.pc++;
    const responses = [
      'router_requirements phase\nforgot to write json\n',
      'must not reach\n'
    ];
    const text = responses[(global.pc - 1) % responses.length];
    console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text } } } }));";
    let script = format!(
        "#!/usr/bin/env node\n{}\n",
        crate::acp_mock_js("", handler)
    );
    std::fs::write(path, script.as_bytes()).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
#[test]
fn kiss_cov_mock_router_agent_helpers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mock = tmp.path().join("mock");
    write_mock_router_agent(&mock);
    assert!(mock.is_file());
    let fail = tmp.path().join("mock-fail");
    write_mock_router_agent_session_fail(&fail);
    assert!(fail.is_file());
    let bad = tmp.path().join("mock-bad");
    write_mock_router_agent_missing_requirements(&bad);
    assert!(bad.is_file());
}
