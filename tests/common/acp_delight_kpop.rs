use super::acp_tidy_kpop::{acp_mock_kpop_iteration_body, acp_mock_kpop_prompt_preamble};
use super::acp_core::{acp_mock_js, session_update_chunk_line};

const DELIGHT_PLAN_WRITE: &str = r"      const outMatch = promptText.match(/Write a new plan to [`']?([^\s`'\n]+)/);
      if (outMatch) {
        let outRel = outMatch[1].replace(/^\.\//, '');
        const outAbs = path.isAbsolute(outRel) ? outRel : path.join(process.cwd(), outRel);
        fs.mkdirSync(path.dirname(outAbs), { recursive: true });
        fs.writeFileSync(outAbs, '# Delight plan\n\nA delightful improvement.\n', 'utf8');
      }";

const DELIGHT_SOLVED_APPEND: &str = r"          fs.appendFileSync(expPath, '\n## KPOP_SOLVED\n');";

fn acp_mock_delight_iteration_body() -> String {
    acp_mock_kpop_iteration_body()
        .replace(
            "      if (expPath) {",
            &format!("{DELIGHT_PLAN_WRITE}\n      if (expPath) {{"),
        )
        .replace(
            "          fs.appendFileSync(expPath, `\\n## Step ${step} — KPOP mock\\n`);",
            &format!(
                "          fs.appendFileSync(expPath, `\\n## Step ${{step}} — KPOP mock\\n`);\n{DELIGHT_SOLVED_APPEND}"
            ),
        )
}

fn acp_mock_delight_kpop_body(plan_write: &str) -> String {
    acp_mock_delight_iteration_body().replace(DELIGHT_PLAN_WRITE, plan_write)
}

const PLAN_PIPELINE_HANDLER: &str = r"    function resolvePlanPath() {
      const patterns = [
        /plan in [`']([^`']+)[`']/,
        /The file [`']([^`']+)[`']/,
        /Read [`']([^`']+\.md)[`']/,
        /Edit [`']([^`']+)[`']/,
        /append to [`']([^`']+)[`']/i,
      ];
      for (const re of patterns) {
        const m = promptText.match(re);
        if (m) {
          const rel = m[1].replace(/^\.\//, '');
          return path.isAbsolute(rel) ? rel : path.join(process.cwd(), rel);
        }
      }
      return path.join(process.cwd(), 'plan.md');
    }
    if (promptText.includes('**Prompt 3**')) {
      console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: '```markdown\n# Revised\n\nDone.\n```' } } } }));
    } else if (promptText.includes('**Prompt 2**')) {
      const planPath = resolvePlanPath();
      fs.appendFileSync(planPath, '\n\n## DECISIONS\n1. **Verdict:** ok **Evidence:** mock\n');
    } else if (promptText.includes('**Prompt 1b**')) {
      const planPath = resolvePlanPath();
      fs.appendFileSync(planPath, '\n\n## Critique\ncrit\n\n## Open questions\n1. q?\n');
    } else if (promptText.includes('**Prompt 1a**')) {
      const planPath = resolvePlanPath();
      fs.appendFileSync(planPath, 'restated\n');
    }";

fn acp_mock_delight_kpop_script(plan_write: &str) -> String {
    format!(
        "{}\n    if (promptText.match(/Complete up to [`]?(\\d+)[`]? KPOP iterations/)) {{\n{}\n    }} else {{\n{}\n    }}",
        acp_mock_kpop_prompt_preamble(),
        acp_mock_delight_kpop_body(plan_write),
        PLAN_PIPELINE_HANDLER
    )
}

pub fn acp_mock_delight_kpop_steps_js() -> String {
    let done = session_update_chunk_line("agent_message_chunk", r"'delight kpop step\n'");
    acp_mock_js("", &format!("{}\n{done}", acp_mock_delight_kpop_script(DELIGHT_PLAN_WRITE)))
}

pub fn acp_mock_delight_kpop_solved_without_output_js() -> String {
    let done = session_update_chunk_line("agent_message_chunk", r"'delight solved only\n'");
    acp_mock_js("", &format!("{}\n{done}", acp_mock_delight_kpop_script("")))
}

pub fn acp_mock_delight_kpop_empty_output_js() -> String {
    let empty_write = r"      const outMatch = promptText.match(/Write a new plan to [`']?([^\s`'\n]+)/);
      if (outMatch) {
        let outRel = outMatch[1].replace(/^\.\//, '');
        const outAbs = path.isAbsolute(outRel) ? outRel : path.join(process.cwd(), outRel);
        fs.mkdirSync(path.dirname(outAbs), { recursive: true });
        fs.writeFileSync(outAbs, '', 'utf8');
      }";
    let done = session_update_chunk_line("agent_message_chunk", r"'delight empty output\n'");
    acp_mock_js("", &format!("{}\n{done}", acp_mock_delight_kpop_script(empty_write)))
}
