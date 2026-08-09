//! ACP mock that completes the default router workflow (`header.md` → `kpop_common.md` → done).

use super::acp_core::acp_mock_js;

/// Minimal router mock: emits `__MALVIN_DONE__` on `router_a`.
/// Also fulfills write output paths when those strings appear in the prompt.
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
    // router_a references the request on disk (`See user requirements at \`…\``); older
    // turns inlined the body. Scan prompt text plus any referenced request file.
    let scanText = promptText;
    const reqPathMatch = promptText.match(/See user requirements at `([^`]+)`/);
    if (reqPathMatch) {
      try {
        const reqAbs = path.isAbsolute(reqPathMatch[1])
          ? reqPathMatch[1]
          : path.join(process.cwd(), reqPathMatch[1]);
        scanText += '\n' + fs.readFileSync(reqAbs, 'utf8');
      } catch (_) {}
    }
    // Match current write_wrapper.md ("Put the LaTeX source in `…`") and the older
    // "Write LaTeX source to `…`" phrasing so mocks stay compatible across wording tweaks.
    const texMatch = scanText.match(/(?:Put the LaTeX source in|Write LaTeX source to) `([^`]+)`/);
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
      text = 'router_header phase\n';
    } else if (global.pc === 2) {
      text = 'router_kpop_common phase\n';
    } else {
      text = 'router_a phase\n__MALVIN_DONE__\n';
    }
    console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text } } } }));
"#;
    acp_mock_js("", handler)
}
