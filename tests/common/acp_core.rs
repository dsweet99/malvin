pub const ARGV_CAPTURE_PREAMBLE: &str = r"const fs = require('fs');
const capturePath = process.env.MALVIN_CAPTURE_ARGS_PATH;
if (capturePath) {
  fs.writeFileSync(capturePath, process.argv.slice(2).join('\n'));
}
";

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

pub fn session_update_chunk_line(kind: &str, text_expr: &str) -> String {
    format!(
        r"    console.log(JSON.stringify({{ jsonrpc: '2.0', method: 'session/update', params: {{ update: {{ sessionUpdate: '{kind}', content: {{ type: 'text', text: {text_expr} }} }} }} }}));"
    )
}

pub fn acp_mock_code_with_run_dir_js(body: &str) -> String {
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

pub fn chunk_line(text: &str) -> String {
    format!(
        r"      console.log(JSON.stringify({{ jsonrpc: '2.0', method: 'session/update', params: {{ update: {{ sessionUpdate: 'agent_message_chunk', content: {{ type: 'text', text: '{text}\n' }} }} }} }}));"
    )
}

pub fn write_artifact_lgtm() -> String {
    "      fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\\n', 'utf8');".to_string()
}

pub fn write_workspace_lgtm() -> String {
    "      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\\n', 'utf8');".to_string()
}
