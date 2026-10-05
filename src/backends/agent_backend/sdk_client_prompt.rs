use std::path::Path;

use crate::agent_process::{AgentError, AgentFault, CoderPromptOptions};
use crate::nested_budget_scopes::BudgetScopeLayer;

use super::acp_attempt_loop::{BackoffChoice, RetrySpec, retry_until_ok};
use super::backend_lifecycle::backend_lifecycle;
use super::sdk_client::SdkClient;
use super::sdk_client_active::ActiveCoderSession;
use super::sdk_client_session_header::send_bound_session_header;

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

pub(super) async fn teardown_sdk_session_after_transport_error(
    client: &mut SdkClient,
    err: &AgentError,
) {
    if !err.requires_coder_session_teardown() {
        return;
    }
    let forget_agent = backend_lifecycle(client.model.backend).forgets_resume_id_on_busy_teardown()
        && err.fault == AgentFault::CursorBusy;
    let _ = client.end_coder_session().await;
    if forget_agent {
        client.last_agent_id = None;
    }
}

pub(super) async fn force_fresh_agent_for_retry(client: &mut SdkClient) {
    let _ = client.end_coder_session().await;
    client.last_agent_id = None;
    client.header_lifecycle.require_redelivery();
}

async fn run_one(
    client: &mut SdkClient,
    prompt: &str,
    phase: Option<crate::run_timing::TimingPhase>,
) -> Result<(), AgentError> {
    ensure_open_session(client).await?;
    let session = super::sdk_client::live_session(client)
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
    let cwd = super::sdk_client::begun_cwd(client)
        .cloned()
        .ok_or_else(|| AgentError("begin_coder_session was not called".into()))?;
    client.begin_coder_session(&cwd).await?;
    send_bound_session_header(client).await
}

pub(super) fn emit_prompt_stdout(
    client: &SdkClient,
    prompt: &str,
    who: &str,
    opts: &CoderPromptOptions<'_>,
) {
    if client.io.raw_output || client.io.no_tee {
        return;
    }
    let label = opts.stdout_bracket_label.unwrap_or(who);
    crate::output::print_outgoing_prompt_log(who, label);
    if client.io.log_full_outgoing_prompts {
        crate::output::append_outgoing_prompt_log_lines(prompt);
    }
}

pub(super) fn append_prompt_files(
    client: &SdkClient,
    prompt: &str,
    log_path: &Path,
    who: &str,
) -> Result<(), AgentError> {
    if let Some(parent) = log_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let line = format_prompt_line(client, prompt, who);
    append_prompt_log_bytes(log_path, line.as_bytes())?;
    if let Some(run_dir) = client.prompts_log_run_dir.as_ref() {
        let _ = append_prompt_log_bytes(&run_dir.join("prompts.log"), line.as_bytes());
    }
    Ok(())
}

fn append_prompt_log_bytes(path: &Path, bytes: &[u8]) -> Result<(), AgentError> {
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut f| std::io::Write::write_all(&mut f, bytes))
        .map_err(|e| AgentError(format!("prompt log write failed: {e}")))
}

fn format_prompt_line(client: &SdkClient, prompt: &str, who: &str) -> String {
    let mut line = format!("{} {who}\n", crate::time_format::timestamp_now_string());
    if client.io.log_full_outgoing_prompts {
        line.push_str(prompt);
        if !prompt.ends_with('\n') {
            line.push('\n');
        }
    }
    line
}
