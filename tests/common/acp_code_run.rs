use super::acp_core::{
    CONCERNS_PROMPT_MATCH_JS, REVIEW_WRITE_PROMPT_MATCH_JS, acp_mock_code_with_run_dir_js,
    acp_mock_js, chunk_line, code_review_fanout_branches, session_update_chunk_line,
    write_artifact_lgtm, write_artifact_non_lgtm, write_review_prep_output, write_workspace_lgtm,
};

pub fn acp_mock_code_abort_result_after_check_plan_lgtm_js() -> String {
    let lgtm = write_artifact_lgtm();
    let review_tail = code_review_fanout_branches(&chunk_line("reviewed"), &write_artifact_lgtm());
    let body = format!(
        r"    if (promptText.includes('write ONLY the four characters')) {{
{lgtm}
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: after check plan\n', 'utf8');
{check_done}
    }} else if (promptText.includes('Implement the plan in')) {{
{implement}
    }}
{review_tail}",
        check_done = chunk_line("check_plan_done"),
        implement = chunk_line("implement_phase_ran"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_check_plan_tampers_malvin_checks_then_implement_verifies_restore_js() -> String
{
    let lgtm = write_artifact_lgtm();
    let review_tail = code_review_fanout_branches(&chunk_line("reviewed"), &write_artifact_lgtm());
    let body = format!(
        r#"    if (promptText.includes('write ONLY the four characters "LGTM"')) {{
      fs.writeFileSync(path.join(process.cwd(), '.malvin/checks'), 'TAMPERED\n', 'utf8');
{lgtm}
{checked}
    }} else if (promptText.includes('Implement the plan in')) {{
      const c = fs.readFileSync(path.join(process.cwd(), '.malvin/checks'), 'utf8');
      if (c === 'kiss check\n') {{
{implement_ok}
      }} else {{
        fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: malvin_checks leaked into implement\n', 'utf8');
{implement_tampered}
      }}
    }}
{review_tail}"#,
        checked = chunk_line("checked"),
        implement_ok = chunk_line("implement ok"),
        implement_tampered = chunk_line("implement saw tampered malvin_checks"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_check_plan_tampers_kissconfig_then_implement_verifies_restore_js() -> String {
    let lgtm = write_artifact_lgtm();
    let review_tail = code_review_fanout_branches(&chunk_line("reviewed"), &write_artifact_lgtm());
    let body = format!(
        r#"    if (promptText.includes('write ONLY the four characters "LGTM"')) {{
      fs.writeFileSync(path.join(process.cwd(), '.kissconfig'), 'TAMPERED\n', 'utf8');
{lgtm}
{checked}
    }} else if (promptText.includes('Implement the plan in')) {{
      const k = fs.readFileSync(path.join(process.cwd(), '.kissconfig'), 'utf8');
      if (k === 'x') {{
{implement_ok}
      }} else {{
        fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: kissconfig leaked into implement\n', 'utf8');
{implement_tampered}
      }}
    }}
{review_tail}"#,
        checked = chunk_line("checked"),
        implement_ok = chunk_line("implement ok"),
        implement_tampered = chunk_line("implement saw tampered kissconfig"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_review_lgtm_to_artifact_js() -> String {
    let review_tail = code_review_fanout_branches(&chunk_line("lgtm"), &write_artifact_lgtm());
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
{implement}
    }}
{review_tail}",
        implement = chunk_line("implemented"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_review_lgtm_with_abort_js() -> String {
    let lgtm = write_artifact_lgtm();
    let prep = write_review_prep_output();
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\n', 'utf8');
    }} else if (promptText.includes('KPop: Review in-scope code for these problems')) {{
{prep}
    }} else if ({REVIEW_WRITE_PROMPT_MATCH_JS}) {{
{lgtm}
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: review lgtm abort test\n', 'utf8');
    }} else if ({CONCERNS_PROMPT_MATCH_JS}) {{
    }} else {{
    }}"
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_review_writes_workspace_lgtm_js() -> String {
    let review_tail = code_review_fanout_branches(&write_workspace_lgtm(), &write_artifact_lgtm());
    let body = format!(
        r"    if (promptText.includes('Find a discrepancy between the codebase and')) {{
      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\n', 'utf8');
    }}
{review_tail}",
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_workspace_review_only_lgtm_js() -> String {
    let body = format!(
        r"    if (!({CONCERNS_PROMPT_MATCH_JS})) {{
      const workspaceReview = path.join(process.cwd(), 'review.md');
      const runRootReview = path.join(runRoot, '..', '..', 'review.md');
      fs.writeFileSync(workspaceReview, 'LGTM\n', 'utf8');
      fs.writeFileSync(runRootReview, 'LGTM\n', 'utf8');
    }}"
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_streaming_update_js() -> String {
    let prompt = session_update_chunk_line("agent_message_chunk", r"'agent message\n'");
    acp_mock_js("", &prompt)
}

pub fn acp_mock_code_max_loops_never_lgtm_js() -> String {
    let chunk = session_update_chunk_line("agent_message_chunk", r"'agent message\n'");
    let review_tail = code_review_fanout_branches("", &write_artifact_non_lgtm());
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
{chunk}
    }}
{review_tail}",
    );
    acp_mock_code_with_run_dir_js(&body)
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
    let review_tail = code_review_fanout_branches(&chunk_line("reviewed"), &write_artifact_lgtm());
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

#[cfg(test)]
mod tests {
    use super::acp_mock_code_review_writes_workspace_lgtm_js;
    use std::process::Command;

    #[test]
    fn acp_mock_code_review_writes_workspace_lgtm_js_passes_node_syntax_check() {
        let js = acp_mock_code_review_writes_workspace_lgtm_js();
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("mock.js");
        std::fs::write(&path, &js).expect("write mock");
        let out = Command::new("node")
            .arg("--check")
            .arg(&path)
            .output()
            .expect("spawn node");
        assert!(
            out.status.success(),
            "mock must be valid JavaScript: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
