use super::acp_core::{
    acp_mock_code_with_run_dir_js, chunk_line, write_artifact_lgtm, write_workspace_lgtm,
};

pub fn acp_mock_code_abort_result_after_check_plan_lgtm_js() -> String {
    let lgtm = write_artifact_lgtm();
    let body = format!(
        r"    if (promptText.includes('write ONLY the four characters')) {{
{lgtm}
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: after check plan\n', 'utf8');
{check_done}
    }} else if (promptText.includes('Implement the plan in')) {{
{implement}
    }} else {{
{lgtm}
{reviewed}
    }}",
        check_done = chunk_line("check_plan_done"),
        implement = chunk_line("implement_phase_ran"),
        reviewed = chunk_line("reviewed"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_check_plan_tampers_kissconfig_then_implement_verifies_restore_js() -> String {
    let lgtm = write_artifact_lgtm();
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
    }} else {{
{lgtm}
{reviewed}
    }}"#,
        checked = chunk_line("checked"),
        implement_ok = chunk_line("implement ok"),
        implement_tampered = chunk_line("implement saw tampered kissconfig"),
        reviewed = chunk_line("reviewed"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_review_lgtm_to_artifact_js() -> String {
    let lgtm = write_artifact_lgtm();
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
{implement}
    }} else {{
{lgtm}
{reviewed}
    }}",
        implement = chunk_line("implemented"),
        reviewed = chunk_line("lgtm"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_review_lgtm_with_abort_js() -> String {
    let lgtm = write_artifact_lgtm();
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\\n', 'utf8');
    }} else if (promptText.includes('Please review the codebase.')) {{
{lgtm}
      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: review lgtm abort test\\n', 'utf8');
    }} else {{
      // no-op for unexpected prompt shapes
    }}",
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_review_writes_workspace_lgtm_js() -> String {
    let body = format!(
        r"    if (promptText.includes('Find a discrepancy between the codebase and')) {{
      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\\n', 'utf8');
    }} else if (promptText.includes('Please review the codebase.')) {{
{workspace_lgtm}
    }} else if (promptText.includes('Concerns')) {{
    }} else {{
      fs.writeFileSync(path.join(process.cwd(), 'review.md'), 'LGTM\\n', 'utf8');
    }}",
        workspace_lgtm = write_workspace_lgtm(),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_workspace_review_only_lgtm_js() -> String {
    let body = r"    if (!promptText.includes('Concerns')) {{
      const workspaceReview = path.join(process.cwd(), 'review.md');
      const runRootReview = path.join(runRoot, '..', '..', 'review.md');
      fs.writeFileSync(workspaceReview, 'LGTM\\n', 'utf8');
      fs.writeFileSync(runRootReview, 'LGTM\\n', 'utf8');
    }}"
    .to_string();
    acp_mock_code_with_run_dir_js(&body)
}
