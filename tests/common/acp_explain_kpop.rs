use super::acp_tidy_kpop::{acp_mock_kpop_iteration_body, acp_mock_kpop_prompt_preamble};
use super::acp_core::{acp_mock_js, session_update_chunk_line};

const EXPLAIN_OUTPUT_WRITE: &str = r"      let texRel;
      const texMatch = promptText.match(/Write LaTeX source to [`']?([^\s`'\n]+)/);
      if (texMatch) {
        texRel = texMatch[1].replace(/^\.\//, '');
      } else {
        texRel = 'gate_loop_exit.tex';
      }
      const texAbs = path.isAbsolute(texRel) ? texRel : path.join(process.cwd(), texRel);
      fs.mkdirSync(path.dirname(texAbs), { recursive: true });
      fs.writeFileSync(texAbs, '\\documentclass{article}\\begin{document}Explain\\end{document}', 'utf8');
      const pdfAbs = texAbs.replace(/\.tex$/, '.pdf');
      fs.writeFileSync(pdfAbs, '%PDF-1.4 mock', 'utf8');";

const EXPLAIN_EMPTY_PDF_WRITE: &str = r"      let texRel;
      const texMatch = promptText.match(/Write LaTeX source to [`']?([^\s`'\n]+)/);
      if (texMatch) {
        texRel = texMatch[1].replace(/^\.\//, '');
      } else {
        texRel = 'gate_loop_exit.tex';
      }
      const texAbs = path.isAbsolute(texRel) ? texRel : path.join(process.cwd(), texRel);
      fs.mkdirSync(path.dirname(texAbs), { recursive: true });
      fs.writeFileSync(texAbs, '\\documentclass{article}', 'utf8');
      const pdfAbs = texAbs.replace(/\.tex$/, '.pdf');
      fs.writeFileSync(pdfAbs, '', 'utf8');";

const EXPLAIN_PRODUCTS_EXIST_CHECK: &str = r"      function explainProductsExist() {
        let sawTex = false;
        let sawPdf = false;
        const autoMode = promptText.includes('snake_case') || promptText.includes('shortened version of the report');
        const locateMatch = promptText.match(/Locate existing explanation products at [`']?([^\s`'\n]+)[`']? and [`']?([^\s`'\n]+)/);
        const candidates = [];
        if (locateMatch) {
          candidates.push(locateMatch[1].replace(/^\.\//, ''), locateMatch[2].replace(/^\.\//, ''));
        }
        try {
          for (const name of fs.readdirSync(process.cwd())) {
            if (autoMode && /^explain(_\d+)?\.(tex|pdf)$/.test(name)) continue;
            candidates.push(name);
          }
        } catch {}
        for (const rel of candidates) {
          const abs = path.isAbsolute(rel) ? rel : path.join(process.cwd(), rel);
          try {
            const st = fs.statSync(abs);
            if (!st.isFile() || st.size === 0) continue;
            if (String(abs).endsWith('.tex')) sawTex = true;
            if (String(abs).endsWith('.pdf')) sawPdf = true;
          } catch {}
        }
        return sawTex && sawPdf;
      }";

fn acp_mock_explain_phase_script(write_body: &str) -> String {
    format!(
        r"{preamble}
{products}
    if (promptText.includes('Role: explain review') || promptText.includes('judge lack-of-satisfaction')) {{
{kpop_body}
{review_reply}
    }} else if (promptText.includes('Role: explain plan')) {{
{kpop_body}
{plan_reply}
    }} else if (promptText.includes('Role: explain work') || promptText.includes('Write LaTeX source')) {{
{write_body}
{work_reply}
    }}",
        preamble = acp_mock_kpop_prompt_preamble(),
        products = EXPLAIN_PRODUCTS_EXIST_CHECK,
        kpop_body = acp_mock_kpop_iteration_body(),
        review_reply = session_update_chunk_line(
            "agent_message_chunk",
            "(explainProductsExist() ? 'LGTM' : 'Missing non-empty .tex and .pdf products. Write and compile the explanation paper.')",
        ),
        plan_reply = session_update_chunk_line(
            "agent_message_chunk",
            r"'Write LaTeX and compile PDF addressing the review gaps.\n'",
        ),
        write_body = write_body,
        work_reply = session_update_chunk_line("agent_message_chunk", r"'explain work done\n'"),
    )
}

pub fn acp_mock_explain_kpop_steps_js() -> String {
    acp_mock_js("", &acp_mock_explain_phase_script(EXPLAIN_OUTPUT_WRITE))
}

pub fn acp_mock_explain_agent_ran_without_output_js() -> String {
    let script = format!(
        r"{preamble}
    if (promptText.includes('Role: explain review') || promptText.includes('judge lack-of-satisfaction')) {{
{kpop_body}
{review}
    }} else if (promptText.includes('Role: explain plan')) {{
{kpop_body}
{plan}
    }} else if (promptText.includes('Role: explain work') || promptText.includes('Write LaTeX source')) {{
{work}
    }}",
        preamble = acp_mock_kpop_prompt_preamble(),
        kpop_body = acp_mock_kpop_iteration_body(),
        review = session_update_chunk_line(
            "agent_message_chunk",
            r"'Missing products; never LGTM.'",
        ),
        plan = session_update_chunk_line(
            "agent_message_chunk",
            r"'Plan: write products (mock will skip).\n'",
        ),
        work = session_update_chunk_line("agent_message_chunk", r"'explain solved only\n'"),
    );
    acp_mock_js("", &script)
}

pub fn acp_mock_explain_kpop_empty_pdf_js() -> String {
    acp_mock_js("", &acp_mock_explain_phase_script(EXPLAIN_EMPTY_PDF_WRITE))
}

pub fn acp_mock_explain_lgtm_first_review_js() -> String {
    // Test-only: seed products during review so LGTM validation passes, then assert plan/work
    // never run (throw if those roles appear).
    let script = format!(
        r"{preamble}
    if (promptText.includes('Role: explain review') || promptText.includes('judge lack-of-satisfaction')) {{
{kpop_body}
      let texRel = 'docs/paper.tex';
      const locateMatch = promptText.match(/Locate existing explanation products at [`']?([^\s`'\n]+)/);
      if (locateMatch) texRel = locateMatch[1].replace(/^\.\//, '');
      const texAbs = path.isAbsolute(texRel) ? texRel : path.join(process.cwd(), texRel);
      fs.mkdirSync(path.dirname(texAbs), {{ recursive: true }});
      fs.writeFileSync(texAbs, '\\documentclass{{article}}\\begin{{document}}Explain\\end{{document}}', 'utf8');
      fs.writeFileSync(texAbs.replace(/\.tex$/, '.pdf'), '%PDF-1.4 mock', 'utf8');
{lgtm}
    }} else if (promptText.includes('Role: explain plan')) {{
      throw new Error('plan must not run after LGTM');
    }} else if (promptText.includes('Role: explain work') || promptText.includes('Write LaTeX source')) {{
      throw new Error('work must not run after LGTM');
    }}",
        preamble = acp_mock_kpop_prompt_preamble(),
        kpop_body = acp_mock_kpop_iteration_body(),
        lgtm = session_update_chunk_line("agent_message_chunk", r"'LGTM'"),
    );
    acp_mock_js("", &script)
}
