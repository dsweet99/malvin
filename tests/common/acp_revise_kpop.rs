use super::acp_tidy_kpop::{acp_mock_kpop_iteration_body, acp_mock_kpop_prompt_preamble};
use super::acp_core::{acp_mock_js, session_update_chunk_line};

const REVISE_DOC_WRITE: &str = r"      const docMatch = promptText.match(/Revise [`']?([^\s`'\n]+)[`']? in place/);
      if (docMatch) {
        let docRel = docMatch[1].replace(/^\.\//, '');
        const docAbs = path.isAbsolute(docRel) ? docRel : path.join(process.cwd(), docRel);
        fs.mkdirSync(path.dirname(docAbs), { recursive: true });
        fs.writeFileSync(docAbs, '# Revised\n\nClear prose.\n', 'utf8');
      }";

const REVISE_SOLVED_APPEND: &str = r"          fs.appendFileSync(expPath, '\n## KPOP_SOLVED\n');";

fn acp_mock_revise_iteration_body() -> String {
    acp_mock_kpop_iteration_body()
        .replace(
            "      if (expPath) {",
            &format!("{REVISE_DOC_WRITE}\n      if (expPath) {{"),
        )
        .replace(
            "          fs.appendFileSync(expPath, `\\n## Step ${step} — KPOP mock\\n`);",
            &format!(
                "          fs.appendFileSync(expPath, `\\n## Step ${{step}} — KPOP mock\\n`);\n{REVISE_SOLVED_APPEND}"
            ),
        )
}

fn acp_mock_revise_kpop_body(doc_write: &str) -> String {
    acp_mock_revise_iteration_body().replace(REVISE_DOC_WRITE, doc_write)
}

fn acp_mock_revise_kpop_script(doc_write: &str) -> String {
    format!(
        "{}\n    if (promptText.match(/Complete up to [`]?(\\d+)[`]? KPOP iterations/)) {{\n{}\n    }}",
        acp_mock_kpop_prompt_preamble(),
        acp_mock_revise_kpop_body(doc_write)
    )
}

pub fn acp_mock_revise_kpop_steps_js() -> String {
    let done = session_update_chunk_line("agent_message_chunk", r"'revise kpop step\n'");
    acp_mock_js("", &format!("{}\n{done}", acp_mock_revise_kpop_script(REVISE_DOC_WRITE)))
}

const REVISE_DOC_DELETE: &str = r"      const docMatch = promptText.match(/Revise [`']?([^\s`'\n]+)[`']? in place/);
      if (docMatch) {
        let docRel = docMatch[1].replace(/^\.\//, '');
        const docAbs = path.isAbsolute(docRel) ? docRel : path.join(process.cwd(), docRel);
        try { fs.unlinkSync(docAbs); } catch (_) {}
      }";

pub fn acp_mock_revise_kpop_solved_without_output_js() -> String {
    let done = session_update_chunk_line("agent_message_chunk", r"'revise solved only\n'");
    acp_mock_js(
        "",
        &format!("{}\n{done}", acp_mock_revise_kpop_script(REVISE_DOC_DELETE)),
    )
}

pub fn acp_mock_revise_kpop_empty_output_js() -> String {
    let empty_write = r"      const docMatch = promptText.match(/Revise [`']?([^\s`'\n]+)[`']? in place/);
      if (docMatch) {
        let docRel = docMatch[1].replace(/^\.\//, '');
        const docAbs = path.isAbsolute(docRel) ? docRel : path.join(process.cwd(), docRel);
        fs.mkdirSync(path.dirname(docAbs), { recursive: true });
        fs.writeFileSync(docAbs, '', 'utf8');
      }";
    let done = session_update_chunk_line("agent_message_chunk", r"'revise empty output\n'");
    acp_mock_js("", &format!("{}\n{done}", acp_mock_revise_kpop_script(empty_write)))
}
