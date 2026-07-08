use super::acp_tidy_kpop::{
    acp_mock_kpop_block_match_js, acp_mock_kpop_iteration_body, acp_mock_kpop_prompt_preamble,
};
use super::acp_core::{acp_mock_js, session_update_chunk_line};

const PRIORS_REPORT_WRITE: &str = r"      const outMatch = promptText.match(/Write a new priors report to `([^`]+)`/);
      if (outMatch) {
        let outRel = outMatch[1].replace(/^\.\//, '');
        const outAbs = path.isAbsolute(outRel) ? outRel : path.join(process.cwd(), outRel);
        fs.mkdirSync(path.dirname(outAbs), { recursive: true });
        fs.writeFileSync(outAbs, '# Priors\n\n- Use clap Args patterns from delight.\n', 'utf8');
      }";

fn acp_mock_priors_iteration_body() -> String {
    acp_mock_kpop_iteration_body()
        .replace(
            "      if (expPath) {",
            &format!("{PRIORS_REPORT_WRITE}\n      if (expPath) {{"),
        )
}

fn acp_mock_priors_kpop_body(report_write: &str) -> String {
    acp_mock_priors_iteration_body().replace(PRIORS_REPORT_WRITE, report_write)
}

fn acp_mock_priors_kpop_script(report_write: &str) -> String {
    format!(
        "{}\n    if ({}) {{\n{}\n    }}",
        acp_mock_kpop_prompt_preamble(),
        acp_mock_kpop_block_match_js(),
        acp_mock_priors_kpop_body(report_write),
    )
}

pub fn acp_mock_priors_kpop_steps_js() -> String {
    let done = session_update_chunk_line("agent_message_chunk", r"'priors kpop step\n'");
    acp_mock_js("", &format!("{}\n{done}", acp_mock_priors_kpop_script(PRIORS_REPORT_WRITE)))
}

pub fn acp_mock_priors_agent_ran_without_output_js() -> String {
    let done = session_update_chunk_line("agent_message_chunk", r"'priors without output\n'");
    acp_mock_js(
        "",
        &format!(
            "{}\n{done}",
            acp_mock_priors_kpop_script(
                r"      // intentionally omit writing the priors report
"
            ),
        ),
    )
}

pub fn acp_mock_priors_kpop_empty_output_js() -> String {
    let empty_write = r"      const outMatch = promptText.match(/Write a new priors report to `([^`]+)`/);
      if (outMatch) {
        let outRel = outMatch[1].replace(/^\.\//, '');
        const outAbs = path.isAbsolute(outRel) ? outRel : path.join(process.cwd(), outRel);
        fs.mkdirSync(path.dirname(outAbs), { recursive: true });
        fs.writeFileSync(outAbs, '', 'utf8');
      }";
    let done = session_update_chunk_line("agent_message_chunk", r"'priors empty output\n'");
    acp_mock_js("", &format!("{}\n{done}", acp_mock_priors_kpop_script(empty_write)))
}
