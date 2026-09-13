use crate::acp::{AgentError, CoderPromptOptions, backoff_after_agent_failure, retries_noun};

use super::sdk_client::{CoderSessionHeader, SdkClient, begun_cwd, live_session};
use super::sdk_client_prompt::{
    append_prompt_files, emit_prompt_stdout, force_fresh_agent_for_retry,
    teardown_sdk_session_after_transport_error,
};

pub(super) async fn send_bound_session_header(client: &mut SdkClient) -> Result<(), AgentError> {
    if client.header_lifecycle.is_satisfied() {
        return Ok(());
    }
    let Some(header) = client.header_lifecycle.pending_header().cloned() else {
        return Ok(());
    };
    let opts = header_prompt_options(&header.stdout_label);
    emit_prompt_stdout(client, &header.prompt, &header.log_who, &opts);
    append_prompt_files(client, &header.prompt, &header.log_path, &header.log_who)?;
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
    let backoff_ceiling = u32::MAX;
    let mut last_error;
    let mut attempts_used = 0_u32;
    loop {
        attempts_used = attempts_used.saturating_add(1);
        match send_header_once(client, &header.prompt, opts).await {
            Ok(()) => {
                client.record_backend_success();
                client.header_lifecycle.mark_satisfied_keeping_header();
                return Ok(());
            }
            Err(e) => {
                last_error = recover_header_send_failure(client, e).await?;
                if client.record_backend_error(&last_error) {
                    return Err(AgentError(
                        super::backend_error_tracker::format_backend_consecutive_error_message(
                            client.model.backend.label(),
                            &last_error,
                            client.max_acp_retries,
                        ),
                    ));
                }
                if backoff_after_agent_failure(
                    client.timing.as_ref(),
                    &last_error,
                    attempts_used,
                    backoff_ceiling,
                )
                .await?
                {
                    break;
                }
            }
        }
    }
    let retries = attempts_used.saturating_sub(1);
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
