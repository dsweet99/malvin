use super::acp_code_fanout_mocks::review_write_try_counter_body;
use super::acp_core::{
    ARGV_CAPTURE_PREAMBLE, CONCERNS_PROMPT_MATCH_JS, REVIEW_WRITE_PROMPT_MATCH_JS,
    acp_mock_code_with_run_dir_js, acp_mock_js, chunk_line, session_update_chunk_line,
    write_artifact_lgtm, write_artifact_non_lgtm, write_review_prep_output, write_workspace_lgtm,
};

pub fn acp_mock_do_streaming_update_js() -> String {
    let msg = session_update_chunk_line("agent_message_chunk", r"'agent message\n'");
    let thought = session_update_chunk_line("agent_thought_chunk", r"'hidden thought\n'");
    acp_mock_js(ARGV_CAPTURE_PREAMBLE, &format!("{msg}\n{thought}"))
}

pub fn acp_mock_do_streaming_wordy_long_msg_js() -> String {
    let prompt = format!(
        "    const words = Array(8).fill('abcdefghij').join(' ');\n{}",
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

pub fn acp_mock_tidy_fanout_branches(
    spawn_branch: &str,
    review_write_tail: &str,
    between_review_write_and_else: &str,
    else_tail: &str,
) -> String {
    format!(
        r"    if (promptText.includes('KPop: Review in-scope code for these problems')) {{
{spawn_branch}
    }} else if ({REVIEW_WRITE_PROMPT_MATCH_JS}) {{
{review_write_tail}
    }}{between_review_write_and_else} else if ({CONCERNS_PROMPT_MATCH_JS}) {{
    }} else {{
{else_tail}
    }}"
    )
}

pub fn acp_mock_tidy_fanout_body(review_write_tail: &str) -> String {
    acp_mock_tidy_fanout_branches(
        &write_review_prep_output(),
        review_write_tail,
        "",
        &chunk_line("coder"),
    )
}

#[must_use]
pub fn acp_mock_tidy_fanout_lgtm_with_abort_js() -> String {
    let review_tail = format!(
        "{}\n      fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: review lgtm abort test\\n', 'utf8');",
        write_artifact_lgtm()
    );
    let body = acp_mock_tidy_fanout_body(&review_tail);
    acp_mock_code_with_run_dir_js(&body)
}

#[must_use]
pub fn acp_mock_tidy_abort_after_first_coder_turn_js() -> String {
    let review_tail = format!("{}\n      {}", write_artifact_lgtm(), chunk_line("review"));
    let coder = chunk_line("coder");
    let coder_abort = format!(
        r"      const coderTriesPath = path.join(runDir, '.tidy_coder_tries');
      let coderN = 0;
      if (fs.existsSync(coderTriesPath)) {{
        coderN = parseInt(fs.readFileSync(coderTriesPath, 'utf8'), 10);
      }}
      coderN += 1;
      fs.writeFileSync(coderTriesPath, String(coderN), 'utf8');
      if (coderN === 1) {{
        fs.writeFileSync(path.join(runDir, 'result.md'), 'ABORT: tidy implement abort\\n', 'utf8');
      }}
{coder}"
    );
    let body =
        acp_mock_tidy_fanout_branches(&write_review_prep_output(), &review_tail, "", &coder_abort);
    acp_mock_code_with_run_dir_js(&body)
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
pub fn acp_mock_tidy_fanout_non_lgtm_then_lgtm_js() -> String {
    let review_tail = review_write_try_counter_body(
        &write_artifact_non_lgtm(),
        &write_artifact_lgtm(),
        &chunk_line("review"),
    );
    let body = acp_mock_tidy_fanout_body(&review_tail);
    acp_mock_code_with_run_dir_js(&body)
}

#[must_use]
pub fn acp_mock_tidy_review_write_never_writes_artifact_js() -> String {
    let review_tail = format!("{}\n      {}", write_workspace_lgtm(), chunk_line("review"));
    let body = acp_mock_tidy_fanout_body(&review_tail);
    acp_mock_code_with_run_dir_js(&body)
}

#[must_use]
pub fn acp_mock_tidy_review_write_succeeds_on_second_attempt_js() -> String {
    let try_counter = review_write_try_counter_body(
        &write_workspace_lgtm(),
        &write_artifact_lgtm(),
        &chunk_line("review"),
    );
    let body = acp_mock_tidy_fanout_body(&try_counter);
    acp_mock_code_with_run_dir_js(&body)
}

#[must_use]
pub fn acp_mock_tidy_fanout_skips_reviewer_outputs_js() -> String {
    let body = acp_mock_tidy_fanout_branches(
        &chunk_line("reviewer"),
        &write_artifact_lgtm(),
        "",
        &chunk_line("coder"),
    );
    acp_mock_code_with_run_dir_js(&body)
}

#[cfg(test)]
mod tidy_fanout_body_tests {
    use super::CONCERNS_PROMPT_MATCH_JS;
    use super::acp_mock_tidy_fanout_body;

    #[test]
    fn acp_mock_tidy_fanout_body_branches_on_concerns_prompt() {
        let body = acp_mock_tidy_fanout_body("");
        assert!(
            body.contains(CONCERNS_PROMPT_MATCH_JS),
            "tidy fanout ACP mock must branch on concerns prompts like code fanout mocks"
        );
    }
}
