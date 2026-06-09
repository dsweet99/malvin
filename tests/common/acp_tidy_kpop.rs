use super::acp_core::{acp_mock_js, session_update_chunk_line};

const fn acp_mock_kpop_prompt_preamble() -> &'static str {
    r"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';"
}

const fn acp_mock_kpop_iteration_body() -> &'static str {
    r"      const wantMatch = promptText.match(/Complete up to [`]?(\d+)[`]? KPOP iterations/);
      const want = wantMatch ? parseInt(wantMatch[1], 10) : 1;
      const pathMatch = promptText.match(/([^\s`]+\/_kpop\/exp_log_[^\s`]+\.md)/);
      let expPath = null;
      if (pathMatch) {
        let p = pathMatch[1];
        if (p.startsWith('./')) expPath = path.join(process.cwd(), p.slice(2));
        else if (p.startsWith('/')) expPath = p;
        else expPath = path.join(process.cwd(), p);
      } else {
        const targetMatch = promptText.match(/exp_log_[^\s`]+\.md/);
        const target = targetMatch ? targetMatch[0] : null;
        const os = require('os');
        const root = path.join(os.homedir(), '.malvin', 'logs');
        if (target && fs.existsSync(root)) {
          outer: for (const hash of fs.readdirSync(root, { withFileTypes: true }).filter((e) => e.isDirectory())) {
            const bucket = path.join(root, hash.name);
            const runs = fs.readdirSync(bucket, { withFileTypes: true })
              .filter((e) => e.isDirectory())
              .map((e) => e.name)
              .sort()
              .reverse();
            for (const run of runs) {
              const candidate = path.join(bucket, run, '_kpop', target);
              if (fs.existsSync(candidate)) { expPath = candidate; break outer; }
            }
          }
        }
      }
      if (expPath) {
        fs.mkdirSync(path.dirname(expPath), { recursive: true });
        let existing = '';
        try { existing = fs.readFileSync(expPath, 'utf8'); } catch { existing = ''; }
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
      }"
}

fn acp_mock_kpop_steps_body() -> String {
    format!(
        "{}\n    if (promptText.match(/Complete up to [`]?(\\d+)[`]? KPOP iterations/)) {{\n{}\n    }}",
        acp_mock_kpop_prompt_preamble(),
        acp_mock_kpop_iteration_body()
    )
}

pub fn acp_mock_kpop_steps_js(chunk: &str) -> String {
    let done = session_update_chunk_line("agent_message_chunk", chunk);
    acp_mock_js("", &format!("{}\n{done}", acp_mock_kpop_steps_body()))
}

pub fn acp_mock_kpop_steps_with_summarize_js(chunk: &str) -> String {
    let kpop_done = session_update_chunk_line("agent_message_chunk", chunk);
    let summarize_done =
        session_update_chunk_line("agent_message_chunk", r"'SUMMARIZE_OK\n'");
    acp_mock_js(
        "",
        &format!(
            "{}\n    if (promptText.includes('Summarize the activity')) {{\n{summarize_done}\n    }} else if (promptText.match(/Complete up to [`]?(\\d+)[`]? KPOP iterations/)) {{\n{}\n{kpop_done}\n    }}",
            acp_mock_kpop_prompt_preamble(),
            acp_mock_kpop_iteration_body(),
        ),
    )
}

pub fn acp_mock_tidy_kpop_steps_js() -> String {
    acp_mock_kpop_steps_js(r"'tidy kpop step\n'")
}

pub fn acp_mock_code_kpop_steps_js() -> String {
    acp_mock_kpop_steps_js(r"'code kpop step\n'")
}

fn acp_mock_kpop_tamper_dotfile_writes_solved_js(rel: &str) -> String {
    let tamper = format!(
        "              fs.writeFileSync(path.join(process.cwd(), '{rel}'), 'TAMPERED\\n', 'utf8');\n              fs.appendFileSync(expPath, '\\n## KPOP_SOLVED\\n');"
    );
    let iteration = acp_mock_kpop_iteration_body().replace(
        "          fs.appendFileSync(expPath, `\\n## Step ${step} — KPOP mock\\n`);",
        &format!(
            "          fs.appendFileSync(expPath, `\\n## Step ${{step}} — KPOP mock\\n`);\n{tamper}"
        ),
    );
    let body = format!(
        "{}\n    if (promptText.match(/Complete up to [`]?(\\d+)[`]? KPOP iterations/)) {{\n{iteration}\n    }}",
        acp_mock_kpop_prompt_preamble(),
    );
    let done = session_update_chunk_line("agent_message_chunk", r"'kpop tamper solved\n'");
    acp_mock_js("", &format!("{body}\n{done}"))
}

pub fn acp_mock_kpop_tampers_kissconfig_writes_solved_js() -> String {
    acp_mock_kpop_tamper_dotfile_writes_solved_js(".kissconfig")
}

pub fn acp_mock_kpop_tampers_gitignore_writes_solved_js() -> String {
    acp_mock_kpop_tamper_dotfile_writes_solved_js(".gitignore")
}

pub fn acp_mock_kpop_tampers_malvin_checks_writes_solved_js() -> String {
    acp_mock_kpop_tamper_dotfile_writes_solved_js(".malvin/checks")
}

pub fn acp_mock_kpop_abort_tampers_checks_js() -> String {
    let abort_tail = r"        const runDir = expPath.includes('/_kpop/')
          ? expPath.split('/_kpop/')[0]
          : path.dirname(expPath);
        fs.writeFileSync(path.join(process.cwd(), '.malvin/checks'), 'TAMPERED\n', 'utf8');
        fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: kpop tamper abort\n');";
    let body = acp_mock_kpop_steps_body().replace(
        "        for (let i = 1; i <= want; i += 1) {",
        &format!("{abort_tail}\n        for (let i = 1; i <= want; i += 1) {{"),
    );
    let done = session_update_chunk_line("agent_message_chunk", r"'abort\n'");
    acp_mock_js("", &format!("{body}\n{done}"))
}

pub fn acp_mock_code_kpop_abort_result_js() -> String {
    let abort_tail = r"        const runDir = expPath.includes('/_kpop/')
          ? expPath.split('/_kpop/')[0]
          : path.dirname(expPath);
        fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: code kpop stop\n');";
    let body = acp_mock_kpop_steps_body().replace(
        "        for (let i = 1; i <= want; i += 1) {",
        &format!("{abort_tail}\n        for (let i = 1; i <= want; i += 1) {{"),
    );
    let done = session_update_chunk_line("agent_message_chunk", r"'abort\n'");
    acp_mock_js("", &format!("{body}\n{done}"))
}
