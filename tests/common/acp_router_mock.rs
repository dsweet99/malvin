//! ACP mock that completes the default router workflow (requirements → `KPop` → done).

use super::acp_core::acp_mock_js;

/// Minimal router mock: writes `review_requirements.json`, emits `## NO_WORK_REMAINING` for each group.
/// Also fulfills delight/explain output paths when those strings appear in the prompt.
pub fn acp_mock_router_no_work_js() -> String {
    let handler = r#"
    if (!global.pc) global.pc = 0;
    global.pc++;
    const fs = require('fs');
    const path = require('path');
    const promptText = (msg.params && msg.params.prompt)
      ? (Array.isArray(msg.params.prompt)
          ? msg.params.prompt.map(p => (p && p.text) || '').join('\n')
          : String(msg.params.prompt))
      : '';
    if (global.pc === 1) {
      let outPath = null;
      const abs = promptText.match(/(\/[^\s`"']+review_requirements\.json)/);
      const rel = promptText.match(/(\.\/[^\s`"']+review_requirements\.json)/);
      if (abs) outPath = abs[1];
      else if (rel) outPath = rel[1];
      if (outPath) {
        const resolved = path.isAbsolute(outPath) ? outPath : path.resolve(process.cwd(), outPath);
        fs.mkdirSync(path.dirname(resolved), { recursive: true });
        fs.writeFileSync(resolved, JSON.stringify({
          groups: [{ title: 'G1', requirements: ['already done'] }]
        }));
      }
    }
    const pitchMatch = promptText.match(/Write the pitch to `([^`]+)`/);
    if (pitchMatch) {
      let outRel = pitchMatch[1].replace(/^\.\//, '');
      const outAbs = path.isAbsolute(outRel) ? outRel : path.join(process.cwd(), outRel);
      fs.mkdirSync(path.dirname(outAbs), { recursive: true });
      fs.writeFileSync(outAbs, '# Delight pitch\n\nA delightful improvement.\n', 'utf8');
    }
    const texMatch = promptText.match(/Write LaTeX source to `([^`]+)`/);
    if (texMatch) {
      let texRel = texMatch[1].replace(/^\.\//, '');
      const texAbs = path.isAbsolute(texRel) ? texRel : path.join(process.cwd(), texRel);
      fs.mkdirSync(path.dirname(texAbs), { recursive: true });
      fs.writeFileSync(texAbs, '\\documentclass{article}\\begin{document}ok\\end{document}\n', 'utf8');
      const pdfAbs = texAbs.replace(/\.tex$/, '.pdf');
      fs.writeFileSync(pdfAbs, '%PDF-1.4 mock\n', 'utf8');
    }
    let text;
    if (promptText.includes('Write a summary of this entire session')) {
      text = 'router_summarize done\n';
    } else if (global.pc === 1) {
      text = 'router_requirements phase\nwrote review_requirements.json\n';
    } else {
      text = 'router_kpop phase\n## NO_WORK_REMAINING 1\n';
    }
    console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text } } } }));
"#;
    acp_mock_js("", handler)
}
