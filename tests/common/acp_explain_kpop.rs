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

const REVISE_DOC_WRITE: &str = r"      const docMatch = promptText.match(/Revise [`']?([^\s`'\n]+)[`']? in place/);
      if (docMatch) {
        let docRel = docMatch[1].replace(/^\.\//, '');
        const docAbs = path.isAbsolute(docRel) ? docRel : path.join(process.cwd(), docRel);
        fs.mkdirSync(path.dirname(docAbs), { recursive: true });
        fs.writeFileSync(docAbs, '# Revised\n\nClear prose.\n', 'utf8');
      }";

const EXPLAIN_SOLVED_APPEND: &str = r"          fs.appendFileSync(expPath, '\n## KPOP_SOLVED\n');";

fn acp_mock_kpop_iteration_with_preface(preface: &str) -> String {
    acp_mock_kpop_iteration_body()
        .replace(
            "      if (expPath) {",
            &format!("{preface}\n      if (expPath) {{"),
        )
        .replace(
            "          fs.appendFileSync(expPath, `\\n## Step ${step} — KPOP mock\\n`);",
            &format!(
                "          fs.appendFileSync(expPath, `\\n## Step ${{step}} — KPOP mock\\n`);\n{EXPLAIN_SOLVED_APPEND}"
            ),
        )
}

fn acp_mock_explain_iteration_body() -> String {
    acp_mock_kpop_iteration_with_preface(EXPLAIN_OUTPUT_WRITE)
}

fn acp_mock_explain_revise_iteration_body() -> String {
    let preface = format!(
        "      if (promptText.includes('Write LaTeX source to') || promptText.includes('Write LaTeX source and compile')) {{\n{EXPLAIN_OUTPUT_WRITE}\n      }} else if (promptText.match(/Revise [`']?([^\\s`']+)[`']? in place/)) {{\n{REVISE_DOC_WRITE}\n      }}"
    );
    acp_mock_kpop_iteration_with_preface(&preface)
}

fn acp_mock_explain_kpop_body(output_write: &str) -> String {
    acp_mock_explain_iteration_body().replace(EXPLAIN_OUTPUT_WRITE, output_write)
}

fn acp_mock_explain_revise_kpop_body() -> String {
    acp_mock_explain_revise_iteration_body()
}

fn acp_mock_explain_kpop_script(output_write: &str) -> String {
    format!(
        "{}\n    if (promptText.match(/Complete up to [`]?(\\d+)[`]? KPOP iterations/)) {{\n{}\n    }}",
        acp_mock_kpop_prompt_preamble(),
        acp_mock_explain_kpop_body(output_write)
    )
}

fn acp_mock_explain_revise_kpop_script() -> String {
    format!(
        "{}\n    if (promptText.match(/Complete up to [`]?(\\d+)[`]? KPOP iterations/)) {{\n{}\n    }}",
        acp_mock_kpop_prompt_preamble(),
        acp_mock_explain_revise_kpop_body()
    )
}

pub fn acp_mock_explain_kpop_steps_js() -> String {
    let done = session_update_chunk_line("agent_message_chunk", r"'explain kpop step\n'");
    acp_mock_js("", &format!("{}\n{done}", acp_mock_explain_revise_kpop_script()))
}

pub fn acp_mock_explain_kpop_solved_without_output_js() -> String {
    let done = session_update_chunk_line("agent_message_chunk", r"'explain solved only\n'");
    acp_mock_js("", &format!("{}\n{done}", acp_mock_explain_kpop_script("")))
}

pub fn acp_mock_explain_kpop_empty_pdf_js() -> String {
    let empty_write = r"      const texMatch = promptText.match(/Write LaTeX source to [`']?([^\s`'\n]+)/);
      if (texMatch) {
        let texRel = texMatch[1].replace(/^\.\//, '');
        const texAbs = path.isAbsolute(texRel) ? texRel : path.join(process.cwd(), texRel);
        fs.mkdirSync(path.dirname(texAbs), { recursive: true });
        fs.writeFileSync(texAbs, '\\documentclass{article}', 'utf8');
        const pdfAbs = texAbs.replace(/\.tex$/, '.pdf');
        fs.writeFileSync(pdfAbs, '', 'utf8');
      }";
    let done = session_update_chunk_line("agent_message_chunk", r"'explain empty pdf\n'");
    acp_mock_js("", &format!("{}\n{done}", acp_mock_explain_kpop_script(empty_write)))
}
