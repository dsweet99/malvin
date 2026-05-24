pub const REVIEW_WRITE_PROMPT_MATCH_JS: &str =
    "promptText.toLowerCase().includes('write your final review')";

pub const CONCERNS_PROMPT_MATCH_JS: &str = "promptText.includes(\"reviewer's concerns\")";

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
    const runRoot = path.join(process.cwd(), '.malvin', 'logs');
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

pub fn write_artifact_non_lgtm() -> String {
    "      fs.writeFileSync(path.join(runDir, 'review.md'), 'problems\\n', 'utf8');".to_string()
}

pub fn review_write_regression_test_body() -> String {
    r"      const fs = require('fs');
      const path = require('path');
      const testPath = path.join(process.cwd(), 'tests', 'review_write_fanout_regression.rs');
      fs.mkdirSync(path.dirname(testPath), { recursive: true });
      fs.writeFileSync(
        testPath,
        '#[test]\nfn review_write_fanout_exposes_bug() { assert!(false); }\n',
        'utf8'
      );"
    .to_string()
}

pub fn code_review_fanout_writes_regression_test_and_non_lgtm() -> String {
    let prep = write_review_prep_output();
    let write_tail = format!(
        "{}\n      {}\n{}",
        review_write_regression_test_body(),
        write_artifact_non_lgtm(),
        chunk_line("reviewed")
    );
    format!(
        r"    if (promptText.includes('Implement the plan in')) {{
{implement}
    }} else if (promptText.includes('KPop: Review in-scope code for these problems')) {{
{prep}
    }} else if ({REVIEW_WRITE_PROMPT_MATCH_JS}) {{
{write_tail}
    }} else if ({CONCERNS_PROMPT_MATCH_JS}) {{
    }} else {{
      // learn, summary, and other coder prompts
    }}",
        implement = chunk_line("implemented"),
    )
}

pub fn write_workspace_lgtm() -> String {
    "      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\\n', 'utf8');".to_string()
}

pub fn write_review_prep_output() -> String {
    r"      fs.writeFileSync(
        path.join(runDir, 'review_prep.md'),
        '## Reviewer 1: mock\n\nExecutive summary:\nok\n\ntl;dr:\nok\n\nExperiment log:\n_mock.md\n',
        'utf8'
      );"
        .to_string()
}

pub fn acp_mock_code_fanout_skips_reviewer_outputs_js() -> String {
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
{implement}
    }} else if (promptText.includes('KPop: Review in-scope code for these problems')) {{
{reviewer_skip}
    }} else if ({REVIEW_WRITE_PROMPT_MATCH_JS}) {{
{write_lgtm}
    }} else {{
      // learn, summary
    }}",
        implement = chunk_line("implemented"),
        reviewer_skip = chunk_line("skipped"),
        write_lgtm = write_artifact_lgtm(),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn code_review_fanout_branches(reviewed_chunk: &str, review_write_body: &str) -> String {
    let prep = write_review_prep_output();
    format!(
        r"    else if (promptText.includes('KPop: Review in-scope code for these problems')) {{
{prep}
    }} else if ({REVIEW_WRITE_PROMPT_MATCH_JS}) {{
{review_write_body}
{reviewed_chunk}
    }} else if ({CONCERNS_PROMPT_MATCH_JS}) {{
    }} else {{
      // learn, summary, and other coder prompts
    }}"
    )
}

pub fn acp_mock_bug_kpop_solved_js() -> String {
    let body = r"    const fs = require('fs');
    const path = require('path');
    const root = path.join(process.cwd(), '.malvin', 'logs');
    if (fs.existsSync(root)) {
      const runs = fs.readdirSync(root, { withFileTypes: true })
        .filter((e) => e.isDirectory())
        .map((e) => e.name)
        .sort()
        .reverse();
      outer: for (const run of runs) {
        const kpopDir = path.join(root, run, '_kpop');
        if (!fs.existsSync(kpopDir)) continue;
        for (const name of fs.readdirSync(kpopDir)) {
          if (name.startsWith('exp_log_') && name.endsWith('.md')) {
            fs.appendFileSync(path.join(kpopDir, name), '\n## KPOP_SOLVED\n');
            break outer;
          }
        }
      }
    }";
    let done = session_update_chunk_line("agent_message_chunk", r"'kpop solved\n'");
    acp_mock_js("", &format!("{body}\n{done}"))
}

#[cfg(all(unix, target_os = "linux"))]
pub fn acp_mock_kpop_tamper_then_restore_js() -> String {
    let body = r"    const fs = require('fs');
    const path = require('path');
    const kpopAttempts = (typeof this.kpopAttempts === 'undefined') ? 0 : this.kpopAttempts;
    this.kpopAttempts = kpopAttempts + 1;
    const kiss = (() => { try { return fs.readFileSync(path.join(process.cwd(), '.kissconfig'), 'utf8'); } catch { return ''; } })();
    if (kpopAttempts === 0) {
      fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'TAMPERED', 'utf8');
    } else if (kiss !== 'k = 1\n') {
      fs.writeFileSync(path.join(process.cwd(), 'result.md'), 'ABORT: kpop tamper restored incorrectly\n', 'utf8');
    }";
    let done = session_update_chunk_line("agent_message_chunk", r"'kpop prompt done\n'");
    acp_mock_js("", &format!("    {body}\n{done}"))
}

#[cfg(test)]
mod review_write_match_phrase {
    #[test]
    fn review_write_prompt_match_js_contains_malvin_phrase() {
        assert!(
            super::REVIEW_WRITE_PROMPT_MATCH_JS
                .contains(malvin::prompts::REVIEW_WRITE_ACP_MATCH_PHRASE)
        );
    }

    #[test]
    fn concerns_prompt_match_js_contains_malvin_phrase() {
        assert!(
            super::CONCERNS_PROMPT_MATCH_JS.contains(malvin::prompts::CONCERNS_ACP_MATCH_SUBSTRING)
        );
    }
}
