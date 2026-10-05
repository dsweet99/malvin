use crate::agent_process::{AgentError, AgentFault, CoderPromptOptions};
use crate::nested_budget_scopes::BudgetScopeLayer;

use super::acp_attempt_loop::{BackoffChoice, RetrySpec, retry_until_ok};
use super::sdk_client::{CoderSessionHeader, SdkClient, begun_cwd, live_session};
use super::sdk_client_prompt::{
    append_prompt_files, emit_prompt_stdout, force_fresh_agent_for_retry,
    teardown_sdk_session_after_transport_error,
};

#[must_use]
pub const fn session_header_is_satisfied(client: &SdkClient) -> bool {
    client.header_lifecycle.is_satisfied()
}

#[must_use]
pub fn pending_session_header(client: &SdkClient) -> Option<(String, String)> {
    client
        .header_lifecycle
        .pending_header()
        .map(|h| (h.prompt.clone(), h.stdout_label.clone()))
}

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
    let lead = format!("{} SDK header prompt failed", client.model.backend.label());
    let spec = RetrySpec {
        ceiling: BudgetScopeLayer::AcpAttempt.effective_attempt_ceiling(None, false),
        failure_lead: &lead,
        backoff: BackoffChoice::Consult,
    };
    retry_until_ok!(
        client,
        spec,
        |c| send_header_once(c, &header.prompt, opts).await,
        |c, err| recover_header_send_failure(c, err).await
    )?;
    client.header_lifecycle.mark_satisfied_keeping_header();
    Ok(())
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
) -> Result<AgentError, AgentError> {
    if err.fault == AgentFault::OutputCap {
        return Err(err);
    }
    teardown_sdk_session_after_transport_error(client, &err).await;
    force_fresh_agent_for_retry(client).await;
    if let Some(cwd) = begun_cwd(client).cloned()
        && let Err(e) = client.begin_coder_session(&cwd).await
        && e.fault == AgentFault::BackendRetryLimit
    {
        return Err(e);
    }
    Ok(err)
}
