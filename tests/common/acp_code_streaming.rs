use super::acp_core::{acp_mock_js, session_update_chunk_line};

pub fn acp_mock_code_streaming_update_js() -> String {
    let prompt = session_update_chunk_line("agent_message_chunk", r"'agent message\n'");
    acp_mock_js("", &prompt)
}

pub fn acp_mock_code_streaming_bold_markdown_js() -> String {
    let prompt = session_update_chunk_line("agent_message_chunk", r"'**boldline**\n'");
    acp_mock_js("", &prompt)
}

pub fn acp_mock_code_streaming_rich_markdown_js() -> String {
    let heading = session_update_chunk_line("agent_message_chunk", r"'# md-heading-xyz\n'");
    let list = session_update_chunk_line("agent_message_chunk", r"'- md-item-xyz\n'");
    let bold = session_update_chunk_line("agent_message_chunk", r"'**md-bold-xyz**\n'");
    acp_mock_js("", &format!("{heading}\n{list}\n{bold}"))
}

pub fn acp_mock_code_streaming_long_bold_markdown_js() -> String {
    let prompt = format!(
        "    const words = Array(12).fill('wrap-bold-xyz').join(' ');\n{}",
        session_update_chunk_line("agent_message_chunk", r"'**' + words + '**\n'")
    );
    acp_mock_js("", &prompt)
}

pub fn acp_mock_code_abort_after_implement_js() -> String {
    let prompt = r"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    if (promptText.includes('Implement the plan in')) {
      const runRoot = path.join(process.cwd(), '_malvin');
      const runDirNames = fs.readdirSync(runRoot, { withFileTypes: true }).filter((e) => e.isDirectory()).map((e) => e.name).sort();
      fs.writeFileSync(path.join(runRoot, runDirNames[0], 'result.md'), 'ABORT: stop now\n', 'utf8');
      console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: 'implementing\n' } } } }));
    } else {
      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\n', 'utf8');
      console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: 'reviewed\n' } } } }));
    }";
    acp_mock_js("", prompt)
}
