use super::acp_core::{
    CONCERNS_PROMPT_MATCH_JS, REVIEW_WRITE_PROMPT_MATCH_JS, acp_mock_code_with_run_dir_js,
    chunk_line, write_artifact_lgtm, write_artifact_non_lgtm, write_review_prep_output,
    write_workspace_lgtm,
};

fn acp_mock_code_fanout_workspace_pollution_js(review_write_snippet: &str) -> String {
    let prep = format!("{}\n{}", write_review_prep_output(), write_workspace_lgtm());
    let review_tail = format!(
        r"    else if (promptText.includes('KPop: Review the codebase for these problems')) {{
{prep}
    }} else if ({REVIEW_WRITE_PROMPT_MATCH_JS}) {{
{review_write_snippet}
{reviewed}
    }} else if ({CONCERNS_PROMPT_MATCH_JS}) {{
    }} else {{
    }}",
        reviewed = chunk_line("reviewed"),
    );
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
{implement}
    }}
{review_tail}",
        implement = chunk_line("implemented"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_review_write_workspace_only_lgtm_js() -> String {
    let review_tail = super::acp_core::code_review_fanout_branches(
        &chunk_line("reviewed"),
        &write_workspace_lgtm(),
    );
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
{implement}
    }}
{review_tail}",
        implement = chunk_line("implemented"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn review_write_try_counter_body(
    workspace_only: &str,
    artifact_lgtm: &str,
    reviewed: &str,
) -> String {
    review_write_succeeds_on_nth_try_body(2, workspace_only, artifact_lgtm, reviewed)
}

pub fn review_write_succeeds_on_nth_try_body(
    succeed_on_try: usize,
    workspace_only: &str,
    artifact_lgtm: &str,
    reviewed: &str,
) -> String {
    format!(
        r"      const triesPath = path.join(runDir, '.review_write_tries');
      let n = 0;
      if (fs.existsSync(triesPath)) {{
        n = parseInt(fs.readFileSync(triesPath, 'utf8'), 10);
      }}
      n += 1;
      fs.writeFileSync(triesPath, String(n), 'utf8');
      if (n < {succeed_on_try}) {{
{workspace_only}
      }} else {{
{artifact_lgtm}
      }}
{reviewed}"
    )
}

#[must_use]
pub fn acp_mock_tidy_review_write_succeeds_on_third_inner_try_js() -> String {
    let reset = r"      const triesPath = path.join(runDir, '.review_write_tries');
      if (fs.existsSync(triesPath)) fs.unlinkSync(triesPath);";
    let prep = format!("{reset}\n{}", write_review_prep_output());
    let review_tail = review_write_succeeds_on_nth_try_body(
        3,
        &write_workspace_lgtm(),
        &write_artifact_lgtm(),
        &chunk_line("review"),
    );
    let body = format!(
        r"    if (promptText.includes('KPop: Review the codebase for these problems')) {{
{prep}
    }} else if ({REVIEW_WRITE_PROMPT_MATCH_JS}) {{
{review_tail}
    }} else {{
{coder}
    }}",
        coder = chunk_line("coder"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_review_write_succeeds_on_second_review_attempt_js() -> String {
    let prep = write_review_prep_output();
    let try_counter = review_write_try_counter_body(
        &write_workspace_lgtm(),
        &write_artifact_lgtm(),
        &chunk_line("reviewed"),
    );
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
{implement}
    }} else if (promptText.includes('KPop: Review the codebase for these problems')) {{
{prep}
    }} else if ({REVIEW_WRITE_PROMPT_MATCH_JS}) {{
{try_counter}
    }} else if ({CONCERNS_PROMPT_MATCH_JS}) {{
    }} else {{
    }}",
        implement = chunk_line("implemented"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_review_write_never_writes_artifact_js() -> String {
    acp_mock_code_fanout_workspace_pollution_js(&write_workspace_lgtm())
}

pub fn acp_mock_code_fanout_reviewer_pollutes_workspace_js() -> String {
    acp_mock_code_fanout_workspace_pollution_js(&write_artifact_non_lgtm())
}

pub fn acp_mock_code_fanout_workspace_only_lgtm_js() -> String {
    acp_mock_code_fanout_workspace_pollution_js(&write_workspace_lgtm())
}

pub fn acp_mock_code_missing_artifact_recovers_on_outer_review_attempt_js() -> String {
    let prep = write_review_prep_output();
    let review_write_by_attempt = format!(
        r"      const triesPath = path.join(runDir, '.outer_review_write_tries');
      let n = 0;
      if (fs.existsSync(triesPath)) {{
        n = parseInt(fs.readFileSync(triesPath, 'utf8'), 10);
      }}
      n += 1;
      fs.writeFileSync(triesPath, String(n), 'utf8');
      if (n === 1) {{
{workspace_only}
      }} else {{
{artifact_lgtm}
      }}
{reviewed}",
        workspace_only = write_workspace_lgtm(),
        artifact_lgtm = write_artifact_lgtm(),
        reviewed = chunk_line("reviewed"),
    );
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
{implement}
    }} else if (promptText.includes('KPop: Review the codebase for these problems')) {{
{prep}
    }} else if ({REVIEW_WRITE_PROMPT_MATCH_JS}) {{
{review_write_by_attempt}
    }} else if ({CONCERNS_PROMPT_MATCH_JS}) {{
    }} else {{
    }}",
        implement = chunk_line("implemented"),
    );
    acp_mock_code_with_run_dir_js(&body)
}
