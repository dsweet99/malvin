use std::path::{Path, PathBuf};

use crate::acp::{AgentError, backoff_after_agent_failure, retries_noun};
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
    client.record_backend_success();
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

async fn handle_spawn_failure(
    client: &mut SdkClient,
    err: AgentError,
    attempt: u32,
    max_attempts: u32,
) -> Result<(String, bool), AgentError> {
    let last_error = note_spawn_failure(client, err);
    if client.record_backend_error(&last_error) {
        return Err(AgentError(
            crate::agent_backend::backend_error_tracker::format_backend_consecutive_error_message(
                client.model.backend.label(),
                &last_error,
                client.max_acp_retries,
            ),
        ));
    }
    let stop =
        backoff_after_agent_failure(client.timing.as_ref(), &last_error, attempt, max_attempts)
            .await?;
    Ok((last_error, stop))
}

pub(super) async fn spawn_with_retries(
    client: &mut SdkClient,
    cwd: PathBuf,
    thinking: Option<&str>,
) -> Result<bool, AgentError> {
    let resume_agent_id = cursor_resume_id(client);
    let mut last_error;
    let backoff_ceiling = crate::nested_budget_scopes::BudgetScopeLayer::AcpSpawnRetry
        .effective_max_attempts(u32::MAX, false);
    let mut attempts_used = 0_u32;
    loop {
        attempts_used = attempts_used.saturating_add(1);
        match lifecycle(client)
            .spawn_session(
                bridge_spawn_args(client, &cwd, thinking),
                resume_agent_id.as_deref(),
                spawn_service_wire(client).as_deref(),
            )
            .await
        {
            Ok(s) => {
                return Ok(record_spawn_success(
                    client,
                    s,
                    cwd,
                    resume_agent_id.as_deref(),
                ));
            }
            Err(e) => {
                let (err_msg, stop) =
                    handle_spawn_failure(client, e, attempts_used, backoff_ceiling).await?;
                last_error = err_msg;
                if stop {
                    break;
                }
            }
        }
    }
    let retries = attempts_used.saturating_sub(1);
    Err(AgentError(format!(
        "{}-sdk-bridge failed to spawn after {retries} {}. Last error:\n{last_error}",
        client.model.backend.label(),
        retries_noun(retries)
    )))
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

fn note_spawn_failure(client: &mut SdkClient, err: AgentError) -> String {
    let mut last_error = err.message;
    if lifecycle(client).tracks_resume_agent_id() && client.last_agent_id.take().is_some() {
        last_error = format!("{last_error} (resume failed; will create)");
    }
    last_error
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
