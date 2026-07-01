//! Minimal ACP stdin/stdout mock script builder (tests and test harnesses).

pub const MPC_REQUEST_PROMPT_MATCH_JS: &str = "promptText.includes('# MPC Request')";

#[must_use]
pub fn acp_mock_mpc_planner_chunk_js() -> String {
    r"    console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: 'mpc planner done\n' } } } }));".to_string()
}

/// Wrap a prompt handler so MPC planner prompts return immediately without side effects.
#[must_use]
pub fn acp_mock_wrap_handler_with_mpc_fast_path(handler: &str) -> String {
    let chunk = acp_mock_mpc_planner_chunk_js();
    format!(
        r"    let promptText = (((msg.params || {{}}).prompt || [])[0] || {{}}).text || '';
    if ({MPC_REQUEST_PROMPT_MATCH_JS}) {{
{chunk}
    }} else {{
{handler}
    }}"
    )
}

#[must_use]
pub fn acp_mock_js(preamble: &str, prompt_handler: &str) -> String {
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
