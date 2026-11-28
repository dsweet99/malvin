use std::path::Path;

use crate::agent_process::{AgentError, AgentFault, CoderPromptOptions};
use crate::nested_budget_scopes::BudgetScopeLayer;

use super::agent_backend::acp_attempt_loop::{BackoffChoice, RetrySpec, retry_until_ok};
use super::agent_backend::sdk_client::SdkClient;
use super::agent_backend::sdk_client_active::ActiveCoderSession;
use super::agent_backend::sdk_client_prompt_io::{
    append_prompt_files, emit_prompt_stdout, force_fresh_agent_for_retry,
    teardown_sdk_session_after_transport_error,
};
use super::agent_backend::sdk_client_session_header::send_bound_session_header;

impl ActiveCoderSession<'_> {
    pub async fn run_coder_prompt(
        &mut self,
        prompt: &str,
        log_path: &Path,
        who: &str,
        opts: CoderPromptOptions<'_>,
    ) -> Result<(), AgentError> {
        debug_assert!(
            !self.client.coder.is_idle(),
            "ActiveCoderSession must not wrap an idle coder slot"
        );
        emit_prompt_stdout(self.client, prompt, who, &opts);
        append_prompt_files(self.client, prompt, log_path, who)?;
        execute_prompt_with_retries(self.client, prompt, &opts).await
    }
}

async fn execute_prompt_with_retries(
    client: &mut SdkClient,
    prompt: &str,
    opts: &CoderPromptOptions<'_>,
) -> Result<(), AgentError> {
    let single = opts.single_attempt;
    let fresh = opts.fresh_agent_on_retry;
    let phase = opts.llm_phase;
    let lead = format!("{} SDK prompt failed", client.model.backend.label());
    let spec = RetrySpec {
        ceiling: BudgetScopeLayer::AcpAttempt.effective_attempt_ceiling(None, single),
        failure_lead: &lead,
        backoff: if single {
            BackoffChoice::StopWithoutSleep
        } else {
            BackoffChoice::Consult
        },
    };
    retry_until_ok!(
        client,
        spec,
        |c| run_one(c, prompt, phase).await,
        |c, err| recover_prompt_failure(c, err, fresh).await
    )?;
    client.record_backend_success();
    Ok(())
}

async fn recover_prompt_failure(
    client: &mut SdkClient,
    err: AgentError,
    fresh_agent_on_retry: bool,
) -> Result<AgentError, AgentError> {
    if matches!(
        err.fault,
        AgentFault::BackendRetryLimit | AgentFault::OutputCap
    ) {
        return Err(err);
    }
    teardown_sdk_session_after_transport_error(client, &err).await;
    if fresh_agent_on_retry {
        force_fresh_agent_for_retry(client).await;
    }
    Ok(err)
}

async fn run_one(
    client: &mut SdkClient,
    prompt: &str,
    phase: Option<crate::run_timing::TimingPhase>,
) -> Result<(), AgentError> {
    ensure_open_session(client).await?;
    let session = super::agent_backend::sdk_client::live_session(client)
        .ok_or_else(|| AgentError("begin_coder_session was not called".into()))?;
    let started = std::time::Instant::now();
    let result = session.send_prompt(prompt).await;
    if let Some(p) = phase {
        crate::run_timing::record_llm(client.timing.as_ref(), p, started.elapsed());
    }
    result
}

async fn ensure_open_session(client: &mut SdkClient) -> Result<(), AgentError> {
    if client.has_open_coder_session() {
        return Ok(());
    }
    let cwd = super::agent_backend::sdk_client::begun_cwd(client)
        .cloned()
        .ok_or_else(|| AgentError("begin_coder_session was not called".into()))?;
    client.begin_coder_session(&cwd).await?;
    send_bound_session_header(client).await
}
