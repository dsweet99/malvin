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
