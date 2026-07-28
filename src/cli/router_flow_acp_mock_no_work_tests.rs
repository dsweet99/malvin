//! Mock ACP agents that emit `NO_WORK_REMAINING` (and related outer-loop variants).

use super::router_flow_acp_mock_tests::ROUTER_MOCK_SESSION_COUNTS_FILE;

/// Writes requirements, then emits `## NO_WORK_REMAINING 1` (skips work).
#[cfg(unix)]
pub(crate) fn write_mock_router_agent_all_no_work(path: &std::path::Path) {
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
          groups: [{{ title: 'G1', requirements: ['already done'] }}, {{ title: 'G2', requirements: ['also done'] }}]
        }}));
      }}
    }}
    const responses = [
      'router_requirements phase\nwrote review_requirements.json\n',
      '## NO_WORK_REMAINING 1\n## NO_WORK_REMAINING 2\n'
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

/// First outer session emits Group Work; later sessions emit all `NO_WORK_REMAINING`.
#[cfg(unix)]
pub(crate) fn write_mock_router_agent_work_then_no_work(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let script = format!(
        r#"#!/usr/bin/env node
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
    if (state.pc === 1) {{
      let outPath = null;
      const abs = promptText.match(/(\/[^\s`"']+review_requirements\.json)/);
      const rel = promptText.match(/(\.\/[^\s`"']+review_requirements\.json)/);
      if (abs) outPath = abs[1];
      else if (rel) outPath = rel[1];
      if (outPath) {{
        const resolved = path.isAbsolute(outPath) ? outPath : path.resolve(process.cwd(), outPath);
        fs.mkdirSync(path.dirname(resolved), {{ recursive: true }});
        fs.writeFileSync(resolved, JSON.stringify({{
          groups: [{{ title: 'G1', requirements: ['do work'] }}]
        }}));
      }}
    }}
    let text;
    if (state.pc === 1) {{
      text = 'router_requirements phase\n';
    }} else if (state.sessions === 1) {{
      text = '## Group Work 1\nresidual\n';
    }} else {{
      text = '## NO_WORK_REMAINING 1\n';
    }}
    if (state.pc === 3 && state.sessions === 1) {{
      text = 'router_work done\n';
    }}
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

#[cfg(unix)]
#[test]
fn write_mock_no_work_helpers_emit_no_work_remaining() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let no_work = tmp.path().join("mock-no-work");
    write_mock_router_agent_all_no_work(&no_work);
    let body = std::fs::read_to_string(&no_work).expect("read");
    assert!(body.contains("NO_WORK_REMAINING"));
    let work_then = tmp.path().join("mock-work-then");
    write_mock_router_agent_work_then_no_work(&work_then);
    let body2 = std::fs::read_to_string(&work_then).expect("read");
    assert!(body2.contains("Group Work 1"));
    assert!(body2.contains("NO_WORK_REMAINING"));
}
