use super::acp_core::{
    acp_mock_code_with_run_dir_js, chunk_line, write_artifact_lgtm, write_artifact_non_lgtm,
    write_fanout_reviewer_output, write_workspace_lgtm,
};

fn acp_mock_code_fanout_workspace_pollution_js(review_write_snippet: &str) -> String {
    let reviewer = format!(
        "{}\n{}",
        write_fanout_reviewer_output(),
        write_workspace_lgtm()
    );
    let review_tail = format!(
        r"    else if (promptText.includes('Write your executive summary and tl;dr to')) {{
{reviewer}
    }} else if (promptText.includes('Read the files in') && promptText.includes('Rate all of the findings')) {{
{review_write_snippet}
{reviewed}
    }} else if (promptText.includes('Concerns')) {{
    }} else {{
    }}",
        reviewer = reviewer,
        review_write_snippet = review_write_snippet,
        reviewed = chunk_line("reviewed"),
    );
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
{implement}
    }}
{review_tail}",
        implement = chunk_line("implemented"),
        review_tail = review_tail,
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

pub fn acp_mock_code_review_write_succeeds_on_second_review_attempt_js() -> String {
    let reviewer = write_fanout_reviewer_output();
    let body = format!(
        r"    if (promptText.includes('Implement the plan in')) {{
{implement}
    }} else if (promptText.includes('Write your executive summary and tl;dr to')) {{
{reviewer}
    }} else if (promptText.includes('Read the files in') && promptText.includes('Rate all of the findings')) {{
      if (promptText.includes('reviewers_attempt_2')) {{
{artifact_lgtm}
      }} else {{
{workspace_only}
      }}
{reviewed}
    }} else if (promptText.includes('Concerns')) {{
    }} else {{
    }}",
        implement = chunk_line("implemented"),
        reviewer = reviewer,
        workspace_only = write_workspace_lgtm(),
        artifact_lgtm = write_artifact_lgtm(),
        reviewed = chunk_line("reviewed"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

pub fn acp_mock_code_fanout_reviewer_pollutes_workspace_js() -> String {
    acp_mock_code_fanout_workspace_pollution_js(&write_artifact_non_lgtm())
}

pub fn acp_mock_code_fanout_workspace_only_lgtm_js() -> String {
    acp_mock_code_fanout_workspace_pollution_js(&write_workspace_lgtm())
}
