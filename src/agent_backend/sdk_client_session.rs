use std::path::{Path, PathBuf};

use crate::acp::{AgentError, AuthError, backoff_after_agent_failure, retries_noun};
use crate::agent_backend::sdk_session::SdkSession;
use crate::bridge_sdk::{BridgeSpawnArgs, SDK_BRIDGE_MAX_AGE};
use crate::model_id::ModelBackend;

use super::sdk_client::{BegunCoderSession, SdkClient};

/// Outcome of ensuring a coder session is open.
///
/// Encodes the header protocol that used to be a bare `bool`: only [`Fresh`]
/// means callers should send `header.md`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoderSessionEnsure {
    /// A new agent context was created — send `header.md`.
    Fresh,
    /// An open session was reused, or a Cursor bridge resumed a prior `agent_id`.
    Reused,
}

impl CoderSessionEnsure {
    #[must_use]
    pub const fn is_fresh(self) -> bool {
        matches!(self, Self::Fresh)
    }
}

impl SdkClient {
    pub fn ensure_authenticated(&self) -> Result<(), AuthError> {
        match self.model.backend {
            ModelBackend::Cursor => crate::cursor_sdk::ensure_sdk_authenticated(),
            ModelBackend::Pi => crate::pi_sdk::ensure_pi_authenticated(&self.model.canonical()),
            ModelBackend::Codex => crate::codex_sdk::ensure_codex_authenticated(),
        }
    }

    /// Ensure a coder session is open.
    ///
    /// Returns [`CoderSessionEnsure::Fresh`] only when a **fresh** agent context was
    /// created (so callers may send `header.md`). Returns [`CoderSessionEnsure::Reused`]
    /// when an open session was reused, or when a Cursor bridge restart **resumed** a
    /// prior `agent_id` (same conversation).
    pub async fn ensure_coder_session(
        &mut self,
        cwd: &Path,
    ) -> Result<CoderSessionEnsure, AgentError> {
        if sdk_bridge_needs_restart(self) {
            self.end_coder_session().await?;
        }
        if self.has_open_coder_session() {
            return Ok(CoderSessionEnsure::Reused);
        }
        let resumed = begin_coder_session_resumed(self, cwd).await?;
        Ok(if resumed {
            CoderSessionEnsure::Reused
        } else {
            CoderSessionEnsure::Fresh
        })
    }

    pub async fn begin_coder_session(&mut self, cwd: &Path) -> Result<(), AgentError> {
        begin_coder_session_resumed(self, cwd).await.map(|_| ())
    }

    pub async fn end_coder_session(&mut self) -> Result<(), AgentError> {
        let Some(home) = self.coder.as_mut() else {
            return Ok(());
        };
        let Some(s) = home.take_live_session() else {
            return Ok(());
        };
        if matches!(self.model.backend, ModelBackend::Cursor) {
            remember_agent_id_from(self, &s);
        }
        s.shutdown().await?;
        Ok(())
    }
}

#[must_use]
pub(crate) fn sdk_bridge_needs_restart(client: &SdkClient) -> bool {
    super::sdk_client::live_session(client)
        .is_some_and(|s| s.started_at.elapsed() >= SDK_BRIDGE_MAX_AGE)
}

/// Begin a coder session. Returns `true` when Cursor resume attached a prior agent.
async fn begin_coder_session_resumed(client: &mut SdkClient, cwd: &Path) -> Result<bool, AgentError> {
    reject_no_force(client)?;
    if client.has_open_coder_session() {
        return Err(AgentError(format!(
            "{} SDK session is already open",
            client.model.backend.label()
        )));
    }
    let cwd = crate::acp::resolve_acp_session_cwd(cwd)?;
    let thinking = spawn_thinking_wire(client);
    spawn_with_retries(client, cwd, thinking.as_deref()).await
}

fn reject_no_force(client: &SdkClient) -> Result<(), AgentError> {
    crate::acp::require_force(client.io.force)
}

fn cursor_resume_id(client: &SdkClient) -> Option<String> {
    matches!(client.model.backend, ModelBackend::Cursor)
        .then(|| client.last_agent_id.clone())
        .flatten()
}

/// Spawn a bridge session. On success, returns whether the spawn used Cursor resume.
async fn spawn_with_retries(
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
                return Ok(resume_agent_id.is_some());
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

fn spawn_thinking_wire(client: &SdkClient) -> Option<String> {
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

fn spawn_service_wire(client: &SdkClient) -> Option<String> {
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

fn note_spawn_failure(client: &mut SdkClient, err: AgentError) -> String {
    let mut last_error = err.message;
    if matches!(client.model.backend, ModelBackend::Cursor) && client.last_agent_id.take().is_some() {
        last_error = format!("{last_error} (resume failed; will create)");
    }
    last_error
}

fn remember_agent_id_from(client: &mut SdkClient, session: &SdkSession) {
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

#[cfg(test)]
#[path = "sdk_client_session_tests.rs"]
mod sdk_client_session_tests;
