//! ACP mock that records `session/cancel` and `session/prompt` in `rpc_trace`.

use std::path::Path;

pub const ACP_MOCK_RPC_TRACE_JS: &str = r#"const fs = require('fs');
const path = require('path');
const readline = require('readline');
const ROOT = __dirname;
const TRACE = path.join(ROOT, 'rpc_trace');
function trace(line) {
  fs.appendFileSync(TRACE, line + '\n');
}
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
  } else if (mid === 'session/cancel') {
    trace('cancel');
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  } else if (mid === 'session/prompt') {
    trace('prompt');
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: { stopReason: 'end' } }));
  } else if (rid != null) {
    console.log(JSON.stringify({ jsonrpc: '2.0', id: rid, result: {} }));
  }
});
"#;

/// Writable fake `agent acp` that records cancel/prompt RPC order in `rpc_trace`.
pub async fn write_acp_jsonrpc_mock_rpc_trace(path: &Path) {
    super::write_acp_mock_script_with_optional_log(path, None, ACP_MOCK_RPC_TRACE_JS).await;
}
