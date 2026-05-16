use super::acp_core::{
    ARGV_CAPTURE_PREAMBLE, acp_mock_code_with_run_dir_js, acp_mock_js, chunk_line,
    session_update_chunk_line, write_artifact_lgtm, write_artifact_non_lgtm,
    write_fanout_reviewer_output,
};

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

fn acp_mock_tidy_fanout_body(review_write_tail: &str) -> String {
    let reviewer = write_fanout_reviewer_output();
    let coder = chunk_line("coder");
    format!(
        r"    if (promptText.includes('Write your executive summary and tl;dr to')) {{
{reviewer}
    }} else if (promptText.includes('Read the files in') && promptText.includes('Rate all of the findings')) {{
{review_write_tail}
    }} else {{
{coder}
    }}"
    )
}

#[must_use]
pub fn acp_mock_tidy_fanout_lgtm_js() -> String {
    let body = acp_mock_tidy_fanout_body(&format!(
        "{}\n      {}",
        write_artifact_lgtm(),
        chunk_line("review")
    ));
    acp_mock_code_with_run_dir_js(&body)
}

#[must_use]
pub fn acp_mock_tidy_fanout_non_lgtm_js() -> String {
    let body = acp_mock_tidy_fanout_body(&format!(
        "{}\n      {}",
        write_artifact_non_lgtm(),
        chunk_line("review")
    ));
    acp_mock_code_with_run_dir_js(&body)
}

#[must_use]
pub fn acp_mock_tidy_fanout_skips_reviewer_outputs_js() -> String {
    let body = format!(
        r"    if (promptText.includes('Write your executive summary and tl;dr to')) {{
      {}
    }} else if (promptText.includes('Read the files in') && promptText.includes('Rate all of the findings')) {{
      {}
    }} else {{
      {}
    }}",
        chunk_line("reviewer"),
        write_artifact_lgtm(),
        chunk_line("coder"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

