//! ACP mocks that stream Cursor HTTP/2 PING `RetriableError`s during requirements.

/// Requirements turn that streams a Cursor PING `RetriableError` then `end_turn` (no JSON write).
#[cfg(unix)]
pub(crate) fn write_mock_router_agent_requirements_ping_timeout(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let handler = r"    const text = 'Error: RetriableError: [unavailable] PING timed out';
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

/// First requirements attempt streams PING (no write); after respawn, writes JSON and finishes the router.
#[cfg(unix)]
pub(crate) fn write_mock_router_agent_requirements_ping_then_ok(path: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;

    let script = r#"#!/usr/bin/env node
const fs = require('fs');
const path = require('path');
const statePath = path.resolve(process.cwd(), '.malvin_router_ping_retry_state.json');
function loadState() {
  try { return JSON.parse(fs.readFileSync(statePath, 'utf8')); } catch (e) { return { pinged: false, phase: 0 }; }
}
function saveState(s) { try { fs.writeFileSync(statePath, JSON.stringify(s)); } catch (e) {} }
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
    const promptText = (msg.params && msg.params.prompt)
      ? (Array.isArray(msg.params.prompt)
          ? msg.params.prompt.map(p => (p && p.text) || '').join('\n')
          : String(msg.params.prompt))
      : '';
    const state = loadState();
    if (!state.pinged) {
      state.pinged = true;
      saveState(state);
      const text = 'Error: RetriableError: [unavailable] PING timed out';
      console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text } } } }));
      console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { stopReason: 'end' } }));
      return;
    }
    state.phase += 1;
    saveState(state);
    if (state.phase === 1) {
      let outPath = null;
      const abs = promptText.match(/(\/[^\s`"']+review_requirements\.json)/);
      const rel = promptText.match(/(\.\/[^\s`"']+review_requirements\.json)/);
      if (abs) outPath = abs[1];
      else if (rel) outPath = rel[1];
      if (outPath) {
        const resolved = path.isAbsolute(outPath) ? outPath : path.resolve(process.cwd(), outPath);
        fs.mkdirSync(path.dirname(resolved), { recursive: true });
        fs.writeFileSync(resolved, JSON.stringify({
          groups: []
        }));
      }
    }
    // Zero groups: requirements then work only (keeps retry coverage under the 1.5s budget).
    const responses = [
      'router_requirements phase\nwrote review_requirements.json\n',
      'router_work done\n'
    ];
    const text = responses[Math.min(state.phase - 1, responses.length - 1)];
    console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text } } } }));
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { stopReason: 'end' } }));
  } else if (rid != null) {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});
"#;
    std::fs::write(path, script.as_bytes()).expect("write mock");
    let mut perms = std::fs::metadata(path).expect("meta").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(path, perms).expect("chmod");
}

#[cfg(unix)]
#[test]
fn kiss_cov_ping_mock_router_agent_helpers() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let ping = tmp.path().join("mock-ping");
    write_mock_router_agent_requirements_ping_timeout(&ping);
    assert!(ping.is_file());
    let ping_ok = tmp.path().join("mock-ping-ok");
    write_mock_router_agent_requirements_ping_then_ok(&ping_ok);
    assert!(ping_ok.is_file());
}
