use super::acp_core::{
    acp_mock_code_with_run_dir_js, acp_mock_js, chunk_line, code_review_fanout_branches,
    session_update_chunk_line, write_artifact_review,
};

pub fn acp_mock_code_streaming_update_js() -> String {
    let prompt = session_update_chunk_line("agent_message_chunk", r"'agent message\n'");
    acp_mock_js("", &prompt)
}

pub fn acp_mock_code_streaming_bold_markdown_js() -> String {
    let prompt = session_update_chunk_line("agent_message_chunk", r"'**boldline**\n'");
    acp_mock_js("", &prompt)
}

pub fn acp_mock_code_streaming_bold_markdown_kpop_steps_js() -> String {
    super::acp_tidy_kpop::acp_mock_kpop_writes_solved_js(r"'**boldline**\n'")
}

pub fn acp_mock_code_streaming_rich_markdown_js() -> String {
    let prompt = session_update_chunk_line(
        "agent_message_chunk",
        r"'# md-heading-xyz\n- md-item-xyz\n**md-bold-xyz**\n'",
    );
    acp_mock_js("", &prompt)
}

pub fn acp_mock_code_streaming_rich_markdown_kpop_steps_js() -> String {
    super::acp_tidy_kpop::acp_mock_rich_markdown_kpop_writes_solved_js()
}

pub fn acp_mock_code_streaming_long_bold_markdown_js() -> String {
    let prompt = format!(
        "    const words = Array(4).fill('wrap-bold-xyz').join(' ');\n{}",
        session_update_chunk_line("agent_message_chunk", r"'**' + words + '**\n'")
    );
    acp_mock_js("", &prompt)
}

pub fn acp_mock_code_abort_after_implement_js() -> String {
    let review_tail =
        code_review_fanout_branches(&chunk_line("reviewed"), &write_artifact_review());
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: stop now\n', 'utf8');
{implement}
    }}
{review_tail}",
        implement = chunk_line("implementing"),
    );
    acp_mock_code_with_run_dir_js(&body)
}
