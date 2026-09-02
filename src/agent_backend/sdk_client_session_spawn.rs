use std::path::{Path, PathBuf};

use crate::acp::{AgentError, backoff_after_agent_failure, retries_noun};
use crate::bridge_sdk::BridgeSpawnArgs;
use crate::model_id::ModelBackend;

use super::super::sdk_client::{BegunCoderSession, SdkClient};
use super::super::sdk_session::SdkSession;

fn cursor_resume_id(client: &SdkClient) -> Option<String> {
    matches!(client.model.backend, ModelBackend::Cursor)
        .then(|| client.last_agent_id.clone())
        .flatten()
}

/// Spawn a bridge session. On success, returns whether the spawn used Cursor resume.
pub(super) async fn spawn_with_retries(
    client: &mut SdkClient,
    cwd: PathBuf,
    thinking: Option<&str>,
) -> Result<bool, AgentError> {
    let resume_agent_id = cursor_resume_id(client);
    let mut last_error = String::new();
    let max_attempts = client.max_acp_retries;
    let mut attempts_used = 0_u32;
    for attempt in 1..=max_attempts {
        attempts_used = attempt;
        match spawn_for_backend(
            client.model.backend,
            bridge_spawn_args(client, &cwd, thinking),
            resume_agent_id.as_deref(),
            spawn_service_wire(client).as_deref(),
        )
        .await
        {
            Ok(s) => {
                adopt_spawned_session(client, s, cwd);
                let resumed = resume_agent_id.is_some();
                if !resumed {
                    emit_agent_started_log(client);
                }
                return Ok(resumed);
            }
            Err(e) => {
                last_error = note_spawn_failure(client, e);
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
        .filter(|_| matches!(client.model.backend, ModelBackend::Pi | ModelBackend::Codex))
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
        .filter(|_| matches!(client.model.backend, ModelBackend::Codex))
        .map(str::to_string)
}

async fn spawn_for_backend(
    backend: ModelBackend,
    args: BridgeSpawnArgs<'_>,
    resume_agent_id: Option<&str>,
    service: Option<&str>,
) -> Result<SdkSession, AgentError> {
    match backend {
        ModelBackend::Cursor => crate::cursor_sdk::spawn_bridge(args, resume_agent_id)
            .await
            .map(|session| SdkSession::Cursor(Box::new(session))),
        ModelBackend::Pi => crate::pi_sdk::spawn_bridge(args).await,
        ModelBackend::Codex => crate::codex_sdk::spawn_bridge(args, service)
            .await
            .map(|session| SdkSession::Codex(Box::new(session))),
    }
}

fn adopt_spawned_session(client: &mut SdkClient, s: SdkSession, cwd: PathBuf) {
    if matches!(client.model.backend, ModelBackend::Cursor) {
        remember_agent_id_from(client, &s);
    }
    client.coder = Some(BegunCoderSession::Live {
        cwd,
        session: s,
    });
    crate::herdr::notify_reclaim();
}

fn emit_agent_started_log(client: &SdkClient) {
    crate::output::print_stdout_line(crate::output::WHO_A, &client.model.canonical());
}

fn note_spawn_failure(client: &mut SdkClient, err: AgentError) -> String {
    let mut last_error = err.message;
    if matches!(client.model.backend, ModelBackend::Cursor) && client.last_agent_id.take().is_some() {
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
