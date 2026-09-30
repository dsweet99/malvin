use std::path::{Path, PathBuf};

use crate::acp::AgentError;
use crate::nested_budget_scopes::BudgetScopeLayer;

use super::super::acp_attempt_loop::{BackoffChoice, RetrySpec, retry_until_ok};
use crate::bridge_sdk::BridgeSpawnArgs;

use super::super::backend_lifecycle::BackendLifecycle;
use super::super::sdk_client::{BegunCoderSession, SdkClient};
use super::super::sdk_session::SdkSession;

const fn lifecycle(client: &SdkClient) -> BackendLifecycle {
    BackendLifecycle::of(client.model.backend)
}

fn cursor_resume_id(client: &SdkClient) -> Option<String> {
    lifecycle(client)
        .tracks_resume_agent_id()
        .then(|| client.last_agent_id.clone())
        .flatten()
}

fn record_spawn_success(
    client: &mut SdkClient,
    session: SdkSession,
    cwd: PathBuf,
    resume_agent_id: Option<&str>,
) -> bool {
    adopt_spawned_session(client, session, cwd);
    let resumed = resume_agent_id.is_some();
    if resumed {
        client.header_lifecycle.mark_satisfied_keeping_header();
    } else {
        client.header_lifecycle.mark_fresh_spawn();
    }
    if !resumed {
        emit_agent_started_log(client);
    }
    resumed
}

pub(super) async fn spawn_with_retries(
    client: &mut SdkClient,
    cwd: PathBuf,
    thinking: Option<&str>,
) -> Result<bool, AgentError> {
    let resume_agent_id = cursor_resume_id(client);
    let lead = format!(
        "{}-sdk-bridge failed to spawn",
        client.model.backend.label()
    );
    let spec = RetrySpec {
        ceiling: BudgetScopeLayer::AcpAttempt.effective_attempt_ceiling(None, false),
        failure_lead: &lead,
        backoff: BackoffChoice::Consult,
    };
    retry_until_ok!(
        client,
        spec,
        |c| attempt_spawn(c, &cwd, thinking, resume_agent_id.as_deref()).await,
        |c, err| Ok(note_spawn_failure(c, err))
    )
}

async fn attempt_spawn(
    client: &mut SdkClient,
    cwd: &Path,
    thinking: Option<&str>,
    resume_agent_id: Option<&str>,
) -> Result<bool, AgentError> {
    let session = lifecycle(client)
        .spawn_session(
            bridge_spawn_args(client, cwd, thinking),
            resume_agent_id,
            spawn_service_wire(client).as_deref(),
        )
        .await?;
    Ok(record_spawn_success(
        client,
        session,
        cwd.to_path_buf(),
        resume_agent_id,
    ))
}

pub(super) fn spawn_thinking_wire(client: &SdkClient) -> Option<String> {
    client
        .model
        .thinking_param()
        .filter(|_| lifecycle(client).supports_thinking_wire())
        .map(str::to_string)
}

fn bridge_spawn_args<'a>(
    client: &'a SdkClient,
    cwd: &'a Path,
    thinking: Option<&'a str>,
) -> BridgeSpawnArgs<'a> {
    BridgeSpawnArgs {
        cwd,
        model: &client.model,
        thinking,
        io: client.io,
        run_dir: client.prompts_log_run_dir.clone(),
        timing: client.timing.clone(),
    }
}

pub(super) fn spawn_service_wire(client: &SdkClient) -> Option<String> {
    client
        .model
        .service_param()
        .filter(|_| lifecycle(client).supports_service_wire())
        .map(str::to_string)
}

fn adopt_spawned_session(client: &mut SdkClient, s: SdkSession, cwd: PathBuf) {
    if lifecycle(client).tracks_resume_agent_id() {
        remember_agent_id_from(client, &s);
    }
    client.coder = BegunCoderSession::Live { cwd, session: s };
    crate::herdr::notify_reclaim();
}

fn emit_agent_started_log(client: &SdkClient) {
    crate::output::print_stdout_line(crate::output::WHO_A, &client.model.canonical());
}

fn note_spawn_failure(client: &mut SdkClient, err: AgentError) -> AgentError {
    let mut message = err.message;
    if lifecycle(client).tracks_resume_agent_id() && client.last_agent_id.take().is_some() {
        message = format!("{message} (resume failed; will create)");
    }
    AgentError {
        message,
        fault: err.fault,
    }
}

pub(super) fn remember_agent_id_from(client: &mut SdkClient, session: &SdkSession) {
    let Some(bridge) = session.as_cursor() else {
        return;
    };
    let id = bridge
        .agent_id
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    if let Some(id) = id {
        client.last_agent_id = Some(id);
    }
}
