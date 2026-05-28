use super::acp_core::{acp_mock_js, session_update_chunk_line};

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
    acp_mock_do_tampers_dotfile_js(".malvin/checks")
}

pub fn acp_mock_do_tampers_malvin_checks_js_only() -> String {
    let tamper = r"    const fs = require('fs');
    const path = require('path');
    fs.writeFileSync(path.join(process.cwd(), '.malvin/checks'), 'TAMPERED', 'utf8');";
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

pub fn acp_mock_do_tampers_malvin_config_js() -> String {
    acp_mock_do_tampers_dotfile_js(".malvin/config.toml")
}

pub fn acp_mock_do_tampers_malvin_config_js_only() -> String {
    let tamper = r"    const fs = require('fs');
    const path = require('path');
    fs.writeFileSync(path.join(process.cwd(), '.malvin/config.toml'), 'TAMPERED', 'utf8');";
    let msg = session_update_chunk_line("agent_message_chunk", r"'ok\n'");
    acp_mock_js("", &format!("{tamper}\n{msg}"))
}

pub fn acp_mock_do_creates_malvin_config_js() -> String {
    let create = r"    const fs = require('fs');
    const path = require('path');
    fs.mkdirSync(path.join(process.cwd(), '.malvin'), { recursive: true });
    fs.writeFileSync(path.join(process.cwd(), '.malvin/config.toml'), 'CREATED', 'utf8');";
    let msg = session_update_chunk_line("agent_message_chunk", r"'ok\n'");
    acp_mock_js("", &format!("{create}\n{msg}"))
}
