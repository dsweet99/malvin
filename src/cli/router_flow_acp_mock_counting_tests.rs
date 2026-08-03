//! Mock ACP agent that always emits `__MALVIN_DONE__` and accumulates session counts.

use super::router_flow_acp_mock_tests::ROUTER_MOCK_SESSION_COUNTS_FILE;

/// Always done; used with failing `--gates` to force outer restarts.
#[cfg(unix)]
pub(crate) fn write_mock_router_agent_all_no_work_counting(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let script = format!(
        r"#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const countsPath = path.resolve(process.cwd(), '{ROUTER_MOCK_SESSION_COUNTS_FILE}');
const counts = {{ begins: 0, prompts: 0, ends: 0 }};
try {{ Object.assign(counts, JSON.parse(fs.readFileSync(countsPath, 'utf8'))); }} catch (e) {{}}
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
    global.pc = 0;
    console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{ sessionId: 't' + counts.begins }} }}));
  }} else if (mid === 'session/prompt') {{
    counts.prompts += 1;
    flushCounts();
    global.pc = (global.pc || 0) + 1;
    const promptText = (msg.params && msg.params.prompt)
      ? (Array.isArray(msg.params.prompt)
          ? msg.params.prompt.map(p => (p && p.text) || '').join('\n')
          : String(msg.params.prompt))
      : '';
    const responses = [
      'router_header phase\n',
      'router_kpop_common phase\n',
      'router_a phase\n__MALVIN_DONE__\n',
      'router_summarize done\n'
    ];
    if (promptText.includes('Write a summary of this entire session')) {{
      try {{
        const p = path.resolve(process.cwd(), '.malvin_router_mock_summarize_count');
        let n = 0;
        try {{ n = parseInt(fs.readFileSync(p, 'utf8'), 10) || 0; }} catch (e) {{}}
        fs.writeFileSync(p, String(n + 1));
      }} catch (e) {{}}
      const text = 'router_summarize done\n';
      console.log(JSON.stringify({{ jsonrpc: '2.0', method: 'session/update', params: {{ update: {{ sessionUpdate: 'agent_message_chunk', content: {{ type: 'text', text }} }} }} }}));
      console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{ stopReason: 'end' }} }}));
      return;
    }}
    const text = responses[Math.min(global.pc - 1, responses.length - 1)];
    console.log(JSON.stringify({{ jsonrpc: '2.0', method: 'session/update', params: {{ update: {{ sessionUpdate: 'agent_message_chunk', content: {{ type: 'text', text }} }} }} }}));
    console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{ stopReason: 'end' }} }}));
  }} else if (rid != null) {{
    console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{}} }}));
  }}
}});
"
    );
    std::fs::write(path, script.as_bytes()).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
#[test]
fn write_mock_counting_helper_persists_counts_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mock = tmp.path().join("mock-counting");
    write_mock_router_agent_all_no_work_counting(&mock);
    let body = std::fs::read_to_string(&mock).expect("read");
    assert!(body.contains(ROUTER_MOCK_SESSION_COUNTS_FILE));
    assert!(body.contains("__MALVIN_DONE__"));
}
