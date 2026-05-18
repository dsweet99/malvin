/// Repo-relative path (under the workflow working directory) for optional injected style text.
///
/// [`AgentClient::new`](crate::AgentClient::new) seeds the client’s style prompt path with
/// `PathBuf::from(DEFAULT_REPO_STYLE_PROMPT_REL)` so [`AgentClient::run_coder_prompt`](crate::AgentClient::run_coder_prompt)
/// can prepend that file when it exists and repo style injection is enabled.
pub const DEFAULT_REPO_STYLE_PROMPT_REL: &str = "coder_style.md";

/// Read optional repo-local style text (trimmed) with the same rules as coder prompt composition.
///
/// Returns `None` when the file is missing, unreadable, or whitespace-only after trim.
pub(crate) fn read_coder_repo_style_text(style_prompt_path: &Path) -> Option<String> {
    std::fs::read_to_string(style_prompt_path)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|t| !t.is_empty())
}

/// Prefix non-empty trimmed style before `prompt`, matching [`coder_prompt_body_with_optional_repo_style`].
pub(crate) fn prepend_coder_repo_style_to_prompt(
    prompt: &str,
    style_trimmed: Option<&str>,
) -> String {
    style_trimmed
        .filter(|t| !t.is_empty())
        .map_or_else(|| prompt.to_string(), |t| format!("{t}\n\n{prompt}"))
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CoderPromptRepoStyleOpts {
    pub style_on_first_turn: bool,
    pub skip_repo_style: bool,
}

/// Build full prompt text and optional repo style read in one pass (at most one `read_to_string` on the style path).
pub(crate) fn coder_prompt_body_with_optional_repo_style(
    prompt: &str,
    style_opts: CoderPromptRepoStyleOpts,
    style_prompt_path: &Path,
) -> (String, Option<String>) {
    let repo_style =
        if style_opts.style_on_first_turn && !style_opts.skip_repo_style {
            read_coder_repo_style_text(style_prompt_path)
        } else {
            None
        };
    let full_prompt = prepend_coder_repo_style_to_prompt(prompt, repo_style.as_deref());
    (full_prompt, repo_style)
}

/// Apply bounded-retry backoff after a failed attempt, or stop the retry loop.
/// Returns `Ok(true)` when the caller should `break` the attempt loop; `Err` on upgrade-plan short-circuit.
async fn backoff_after_agent_failure(
    timing: Option<&std::sync::Arc<std::sync::Mutex<crate::run_timing::RunTiming>>>,
    last_error: &str,
    attempt: u32,
) -> Result<bool, AgentError> {
    warn!(attempt, error = %last_error, "agent acp attempt failed");
    match plan_agent_retry(last_error, attempt) {
        Err(e) => Err(e),
        Ok(AgentRetryOutcome::StopRetrying) => Ok(true),
        Ok(AgentRetryOutcome::Sleep(d)) => {
            crate::run_timing::record_backoff(timing, d);
            tokio_sleep(d).await;
            Ok(false)
        }
    }
}
