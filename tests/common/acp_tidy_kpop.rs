use super::acp_core::{acp_mock_js, session_update_chunk_line};

const fn acp_mock_kpop_steps_body() -> &'static str {
    r"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    const wantMatch = promptText.match(/Complete up to [`]?(\d+)[`]? KPOP iterations/);
    const want = wantMatch ? parseInt(wantMatch[1], 10) : 1;
    const targetMatch = promptText.match(/exp_log_[^\s`]+\.md/);
    const target = targetMatch ? targetMatch[0] : null;
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
        const names = target ? [target] : fs.readdirSync(kpopDir);
        for (const name of names) {
          if (!name.startsWith('exp_log_') || !name.endsWith('.md')) continue;
          const expPath = path.join(kpopDir, name);
          let existing = '';
          try { existing = fs.readFileSync(expPath, 'utf8'); } catch { continue; }
          const stepRe = /^## Step (\d+) — KPOP/m;
          let maxStep = 0;
          for (const line of existing.split('\n')) {
            const m = line.match(stepRe);
            if (m) maxStep = Math.max(maxStep, parseInt(m[1], 10));
          }
          for (let i = 1; i <= want; i += 1) {
            const step = maxStep + i;
            fs.appendFileSync(expPath, `\n## Step ${step} — KPOP mock\n`);
          }
          break outer;
        }
      }
    }"
}

pub fn acp_mock_kpop_steps_js(chunk: &str) -> String {
    let done = session_update_chunk_line("agent_message_chunk", chunk);
    acp_mock_js("", &format!("{}\n{done}", acp_mock_kpop_steps_body()))
}

pub fn acp_mock_tidy_kpop_steps_js() -> String {
    acp_mock_kpop_steps_js(r"'tidy kpop step\n'")
}

pub fn acp_mock_code_kpop_steps_js() -> String {
    acp_mock_kpop_steps_js(r"'code kpop step\n'")
}

pub fn acp_mock_code_kpop_abort_result_js() -> String {
    let abort_tail = r"        const resultPath = path.join(root, run, 'result.md');
        fs.writeFileSync(resultPath, 'ABORT: code kpop stop\n');";
    let body = acp_mock_kpop_steps_body().replace("break outer;", abort_tail);
    let done = session_update_chunk_line("agent_message_chunk", r"'abort\n'");
    acp_mock_js("", &format!("{body}\n{done}"))
}
