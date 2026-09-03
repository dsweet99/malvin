use crate::acp::{AgentError, CoderPromptOptions, backoff_after_agent_failure, retries_noun};

use super::sdk_client::{CoderSessionHeader, SdkClient, begun_cwd, live_session};
use super::sdk_client_prompt::{
    append_prompt_files, emit_prompt_stdout, force_fresh_agent_for_retry,
    teardown_sdk_session_after_transport_error,
};

/// Send the bound spawn header once per fresh agent context.
pub(super) async fn send_bound_session_header(client: &mut SdkClient) -> Result<(), AgentError> {
    if client.header_delivered {
        return Ok(());
    }
    let Some(header) = client.session_header.clone() else {
        return Ok(());
    };
    let opts = header_prompt_options(&header.stdout_label);
    emit_prompt_stdout(client, &header.prompt, "header", &opts);
    append_prompt_files(client, &header.prompt, &header.log_path, "header")?;
    try_send_header_with_retries(client, &header, &opts).await
}

fn header_prompt_options<'a>(stdout_label: &'a str) -> CoderPromptOptions<'a> {
    CoderPromptOptions {
        llm_phase: Some(crate::run_timing::TimingPhase::Implement),
        stdout_bracket_label: Some(stdout_label),
        fresh_agent_on_retry: true,
        ..Default::default()
    }
}

#[cfg(test)]
#[must_use]
pub(crate) fn header_prompt_options_for_test<'a>() -> CoderPromptOptions<'a> {
    header_prompt_options(crate::prompts::header_prompt_file())
}

async fn try_send_header_with_retries(
    client: &mut SdkClient,
    header: &CoderSessionHeader,
    opts: &CoderPromptOptions<'_>,
) -> Result<(), AgentError> {
    let max_attempts = client.max_acp_retries;
    let mut last_error = String::new();
    for attempt in 1..=max_attempts {
        match send_header_once(client, &header.prompt, opts).await {
            Ok(()) => {
                client.header_delivered = true;
                return Ok(());
            }
            Err(e) => {
                last_error = recover_header_send_failure(client, e).await?;
                if backoff_after_agent_failure(
                    client.timing.as_ref(),
                    &last_error,
                    attempt,
                    max_attempts,
                )
                .await?
                {
                    break;
                }
            }
        }
    }
    let retries = max_attempts.saturating_sub(1);
    Err(AgentError(format!(
        "{} SDK header prompt failed after {retries} {}. Last error:\n{last_error}",
        client.model.backend.label(),
        retries_noun(retries)
    )))
}

async fn send_header_once(
    client: &mut SdkClient,
    prompt: &str,
    opts: &CoderPromptOptions<'_>,
) -> Result<(), AgentError> {
    let started = std::time::Instant::now();
    let result = match live_session(client) {
        Some(session) => session.send_prompt(prompt).await,
        None => Err(AgentError("begin_coder_session was not called".into())),
    };
    if let Some(p) = opts.llm_phase {
        crate::run_timing::record_llm(client.timing.as_ref(), p, started.elapsed());
    }
    result
}

async fn recover_header_send_failure(
    client: &mut SdkClient,
    err: AgentError,
) -> Result<String, AgentError> {
    teardown_sdk_session_after_transport_error(client, &err).await;
    force_fresh_agent_for_retry(client).await;
    if let Some(cwd) = begun_cwd(client).cloned() {
        let _ = client.begin_coder_session(&cwd).await;
    }
    Ok(err.message)
}
