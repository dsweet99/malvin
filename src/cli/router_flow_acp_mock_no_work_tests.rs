//! Mock ACP agents that emit `__MALVIN_DONE__` (and related outer-loop variants).

use super::router_flow_acp_mock_tests::{
    write_mock_router_agent_with_responses, ROUTER_MOCK_SESSION_COUNTS_FILE,
};

/// `header.md` → `kpop_common.md` → `router_a.md` with `__MALVIN_DONE__` (skips `router_b`).
#[cfg(unix)]
pub(crate) fn write_mock_router_agent_all_no_work(path: &std::path::Path) {
    write_mock_router_agent_with_responses(
        path,
        r"[
      'router_header phase\n',
      'router_kpop_common phase\n',
      'router_a phase\n__MALVIN_DONE__\n',
      'router_summarize done\n'
    ]",
        "",
    );
}

/// First outer session: not done + `router_b`; later sessions: `__MALVIN_DONE__`.
#[cfg(unix)]
pub(crate) fn write_mock_router_agent_work_then_no_work(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let script = format!(
        r"#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const countsPath = path.resolve(process.cwd(), '{ROUTER_MOCK_SESSION_COUNTS_FILE}');
const statePath = path.resolve(process.cwd(), '.malvin_router_outer_state.json');
function loadState() {{
  try {{ return JSON.parse(fs.readFileSync(statePath, 'utf8')); }} catch (e) {{ return {{ sessions: 0 }}; }}
}}
function saveState(s) {{ try {{ fs.writeFileSync(statePath, JSON.stringify(s)); }} catch (e) {{}} }}
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
    const state = loadState();
    state.sessions += 1;
    state.pc = 0;
    saveState(state);
    console.log(JSON.stringify({{ jsonrpc: '2.0', id: rid, result: {{ sessionId: 't' + state.sessions }} }}));
  }} else if (mid === 'session/prompt') {{
    counts.prompts += 1;
    flushCounts();
    const state = loadState();
    state.pc = (state.pc || 0) + 1;
    saveState(state);
    const promptText = (msg.params && msg.params.prompt)
      ? (Array.isArray(msg.params.prompt)
          ? msg.params.prompt.map(p => (p && p.text) || '').join('\n')
          : String(msg.params.prompt))
      : '';
    let text;
    if (promptText.includes('Write a summary of this entire session')) {{
      try {{
        const p = path.resolve(process.cwd(), '.malvin_router_mock_summarize_count');
        let n = 0;
        try {{ n = parseInt(fs.readFileSync(p, 'utf8'), 10) || 0; }} catch (e) {{}}
        fs.writeFileSync(p, String(n + 1));
      }} catch (e) {{}}
      text = 'router_summarize done\n';
    }} else if (state.pc === 1) {{
      text = 'router_header phase\n';
    }} else if (state.pc === 2) {{
      text = 'router_kpop_common phase\n';
    }} else if (state.sessions === 1) {{
      if (state.pc === 3) {{
        text = 'router_a phase\nnot done\n';
      }} else {{
        text = 'router_b done\n';
      }}
    }} else if (state.pc === 3) {{
      text = 'router_a phase\n__MALVIN_DONE__\n';
    }} else {{
      text = 'unexpected prompt\n';
    }}
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
fn write_mock_no_work_helpers_emit_malvin_done() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let no_work = tmp.path().join("mock-no-work");
    write_mock_router_agent_all_no_work(&no_work);
    let body = std::fs::read_to_string(&no_work).expect("read");
    assert!(body.contains("__MALVIN_DONE__"));
    let work_then = tmp.path().join("mock-work-then");
    write_mock_router_agent_work_then_no_work(&work_then);
    let body2 = std::fs::read_to_string(&work_then).expect("read");
    assert!(body2.contains("router_b done"));
    assert!(body2.contains("__MALVIN_DONE__"));
}
