use super::acp_core::{acp_mock_code_with_run_dir_js, acp_mock_js, session_update_chunk_line};

pub const fn acp_mock_kpop_prompt_preamble() -> &'static str {
    r"    const fs = require('fs');
    const path = require('path');
    let promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    let reqRel = null;
    const legacyReqMatch = promptText.match(/User request \(read this file\):\s*\n\n`([^`]+)`/);
    if (legacyReqMatch) {
      reqRel = legacyReqMatch[1].replace(/^\.\//, '');
    } else {
      const mpcReqMatch = promptText.match(/User request[^`]*`([^`]+)`/i);
      if (mpcReqMatch) reqRel = mpcReqMatch[1].replace(/^\.\//, '');
    }
    if (reqRel) {
      const reqAbs = path.isAbsolute(reqRel) ? reqRel : path.join(process.cwd(), reqRel);
      try {
        promptText += '\n' + fs.readFileSync(reqAbs, 'utf8');
      } catch {}
    }"
}

pub const fn acp_mock_kpop_block_match_js() -> &'static str {
    "promptText.includes('_kpop/exp_log_')"
}

pub const fn acp_mock_kpop_iteration_body() -> &'static str {
    r"      const want = 1;
      function resolvePromptPath(relOrAbs) {
        if (relOrAbs.startsWith('./')) return path.join(process.cwd(), relOrAbs.slice(2));
        if (relOrAbs.startsWith('/')) return relOrAbs;
        return path.join(process.cwd(), relOrAbs);
      }
      const pathMatch = promptText.match(/([^\s`]+\/_kpop\/exp_log_[^\s`]+\.md)/);
      let expPath = null;
      if (pathMatch) {
        expPath = resolvePromptPath(pathMatch[1]);
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
        "{}\n    if ({}) {{\n{}\n    }}",
        acp_mock_kpop_prompt_preamble(),
        acp_mock_kpop_block_match_js(),
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
            "{}\n    if (promptText.includes('Summarize the activity')) {{\n{summarize_done}\n    }} else if ({}) {{\n{}\n{kpop_done}\n    }}",
            acp_mock_kpop_prompt_preamble(),
            acp_mock_kpop_block_match_js(),
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

pub fn acp_mock_kpop_writes_solved_js(chunk: &str) -> String {
    let solved = "              fs.appendFileSync(expPath, '\\n## KPOP_SOLVED\\n');";
    let iteration = acp_mock_kpop_iteration_body().replace(
        "          fs.appendFileSync(expPath, `\\n## Step ${step} — KPOP mock\\n`);",
        &format!(
            "          fs.appendFileSync(expPath, `\\n## Step ${{step}} — KPOP mock\\n`);\n{solved}"
        ),
    );
    let body = format!(
        "{}\n    if (promptText.match(/Complete up to [`]?(\\d+)[`]? KPOP iterations/)) {{\n{iteration}\n    }}",
        acp_mock_kpop_prompt_preamble(),
    );
    let done = session_update_chunk_line("agent_message_chunk", chunk);
    acp_mock_js("", &format!("{body}\n{done}"))
}

pub fn acp_mock_rich_markdown_kpop_writes_solved_js() -> String {
    acp_mock_kpop_writes_solved_js(
        r"'# md-heading-xyz\n- md-item-xyz\n**md-bold-xyz**\n'",
    )
}

fn acp_mock_kpop_tamper_dotfile_writes_solved_js(rel: &str) -> String {
    let tamper = format!(
        "              fs.writeFileSync(path.join(process.cwd(), '{rel}'), 'TAMPERED\\n', 'utf8');"
    );
    let iteration = acp_mock_kpop_iteration_body().replace(
        "          fs.appendFileSync(expPath, `\\n## Step ${step} — KPOP mock\\n`);",
        &format!(
            "          fs.appendFileSync(expPath, `\\n## Step ${{step}} — KPOP mock\\n`);\n{tamper}"
        ),
    );
    let body = format!(
        "{}\n    if ({}) {{\n{iteration}\n    }}",
        acp_mock_kpop_prompt_preamble(),
        acp_mock_kpop_block_match_js(),
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

pub fn acp_mock_kpop_tampers_home_malvin_config_writes_solved_js() -> String {
    let tamper = r"              const os = require('os');
              fs.writeFileSync(path.join(os.homedir(), '.malvin_home', 'config.toml'), 'TAMPERED\n', 'utf8');";
    let iteration = acp_mock_kpop_iteration_body().replace(
        "          fs.appendFileSync(expPath, `\\n## Step ${step} — KPOP mock\\n`);",
        &format!(
            "          fs.appendFileSync(expPath, `\\n## Step ${{step}} — KPOP mock\\n`);\n{tamper}"
        ),
    );
    let body = format!(
        "{}\n    if ({}) {{\n{iteration}\n    }}",
        acp_mock_kpop_prompt_preamble(),
        acp_mock_kpop_block_match_js(),
    );
    let done = session_update_chunk_line("agent_message_chunk", r"'kpop home config tamper solved\n'");
    acp_mock_js("", &format!("{body}\n{done}"))
}

pub fn acp_mock_kpop_abort_tampers_checks_js() -> String {
    acp_mock_immediate_abort_tampers_checks_js("kpop tamper abort")
}

pub fn acp_mock_code_kpop_abort_result_js() -> String {
    acp_mock_immediate_abort_result_js("code kpop stop")
}

pub fn acp_mock_immediate_abort_result_js(message: &str) -> String {
    let body = format!(
        r"    if (promptText.includes('_kpop/exp_log_')) {{
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: {message}\n');
    }}"
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_immediate_abort_tampers_checks_js(message: &str) -> String {
    let body = format!(
        r"    if (promptText.includes('_kpop/exp_log_')) {{
      fs.writeFileSync(path.join(process.cwd(), '.malvin/checks'), 'TAMPERED\n', 'utf8');
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: {message}\n');
    }}"
    );
    acp_mock_code_with_run_dir_js(&body)
}
