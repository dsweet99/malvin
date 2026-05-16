use super::acp_core::{ARGV_CAPTURE_PREAMBLE, acp_mock_js, session_update_chunk_line};

pub fn acp_mock_do_streaming_update_js() -> String {
    let msg = session_update_chunk_line("agent_message_chunk", r"'agent message\n'");
    let thought = session_update_chunk_line("agent_thought_chunk", r"'hidden thought\n'");
    acp_mock_js(ARGV_CAPTURE_PREAMBLE, &format!("{msg}\n{thought}"))
}

pub fn acp_mock_do_streaming_wordy_long_msg_js() -> String {
    let prompt = format!(
        "    const words = Array(15).fill('abcdefghij').join(' ');\n{}",
        session_update_chunk_line("agent_message_chunk", r"words + '\n'")
    );
    acp_mock_js("", &prompt)
}

pub fn acp_mock_do_streaming_long_agent_msg_js() -> String {
    let prompt = format!(
        "    const long = 'a'.repeat(120);\n{}",
        session_update_chunk_line("agent_message_chunk", r"long + '\n'")
    );
    acp_mock_js("", &prompt)
}

fn acp_mock_do_tampers_dotfile_js(file_name: &str) -> String {
    let tamper = format!(
        "    const fs = require('fs');
    const path = require('path');
    fs.writeFileSync(path.join(process.cwd(), '{file_name}'), 'TAMPERED', 'utf8');"
    );
    let msg = session_update_chunk_line("agent_message_chunk", r"'ok\n'");
    let thought = session_update_chunk_line("agent_thought_chunk", r"'t\n'");
    acp_mock_js("", &format!("{tamper}\n{msg}\n{thought}"))
}

pub fn acp_mock_do_tampers_kissconfig_js() -> String {
    acp_mock_do_tampers_dotfile_js(".kissconfig")
}

pub fn acp_mock_do_tampers_kissconfig_js_only() -> String {
    let tamper = r"    const fs = require('fs');
    const path = require('path');
    fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'TAMPERED', 'utf8');";
    let msg = session_update_chunk_line("agent_message_chunk", r"'ok\n'");
    acp_mock_js("", &format!("{tamper}\n{msg}"))
}

pub fn acp_mock_do_creates_kissconfig_js() -> String {
    let create = r"    const fs = require('fs');
    const path = require('path');
    fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'CREATED', 'utf8');";
    let msg = session_update_chunk_line("agent_message_chunk", r"'ok\n'");
    acp_mock_js("", &format!("{create}\n{msg}"))
}

pub fn acp_mock_do_tampers_malvin_checks_js() -> String {
    acp_mock_do_tampers_dotfile_js(".malvin_checks")
}

pub fn acp_mock_do_tampers_malvin_checks_js_only() -> String {
    let tamper = r"    const fs = require('fs');
    const path = require('path');
    fs.writeFileSync(path.join(process.cwd(), '.malvin_checks'), 'TAMPERED', 'utf8');";
    let msg = session_update_chunk_line("agent_message_chunk", r"'ok\n'");
    acp_mock_js("", &format!("{tamper}\n{msg}"))
}

pub fn acp_mock_do_tampers_kissignore_js() -> String {
    acp_mock_do_tampers_dotfile_js(".kissignore")
}

pub fn acp_mock_do_tampers_kissignore_js_only() -> String {
    let tamper = r"    const fs = require('fs');
    const path = require('path');
    fs.writeFileSync(path.join(process.cwd(), '.kissignore'), 'TAMPERED', 'utf8');";
    let msg = session_update_chunk_line("agent_message_chunk", r"'ok\n'");
    acp_mock_js("", &format!("{tamper}\n{msg}"))
}

pub fn acp_mock_do_creates_kissignore_js() -> String {
    let create = r"    const fs = require('fs');
    const path = require('path');
    fs.writeFileSync(path.join(process.cwd(), '.kissignore'), 'CREATED', 'utf8');";
    let msg = session_update_chunk_line("agent_message_chunk", r"'ok\n'");
    acp_mock_js("", &format!("{create}\n{msg}"))
}

#[allow(clippy::needless_raw_string_hashes)]
const TIDY_REVIEWER_LGTM_HANDLER: &str = r#"    const fs = require('fs');
    const path = require('path');
    const promptText = (((msg.params || {}).prompt || [])[0] || {}).text || '';
    const runRoot = path.join(process.cwd(), '_malvin');
    const runDirNames = fs.readdirSync(runRoot, { withFileTypes: true }).filter((e) => e.isDirectory()).map((e) => e.name).sort();
    const runDir = path.join(runRoot, runDirNames[runDirNames.length - 1]);
    if (promptText.includes('<!-- malvin:review_tidy_turn_v1 -->')) {
      fs.writeFileSync(path.join(runDir, 'review.md'), 'LGTM\n', 'utf8');
      console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: 'review\n' } } } }));
    } else {
      console.log(JSON.stringify({ jsonrpc: '2.0', method: 'session/update', params: { update: { sessionUpdate: 'agent_message_chunk', content: { type: 'text', text: 'coder\n' } } } }));
    }"#;

#[must_use]
pub fn acp_mock_tidy_reviewer_lgtm_js() -> String {
    acp_mock_js("", TIDY_REVIEWER_LGTM_HANDLER)
}
