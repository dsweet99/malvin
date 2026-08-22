use std::path::{Path, PathBuf};

use crate::acp::{AgentError, AuthError, backoff_after_agent_failure, retries_noun};
use crate::agent_backend::sdk_session::SdkSession;
use crate::bridge_sdk::{BridgeSpawnArgs, SDK_BRIDGE_MAX_AGE};

use super::sdk_client::{BridgeKind, SdkClient};

impl SdkClient {
    pub fn ensure_authenticated(&self) -> Result<(), AuthError> {
        match self.kind {
            BridgeKind::Cursor => crate::cursor_sdk::ensure_sdk_authenticated(),
            BridgeKind::Pi => crate::pi_sdk::ensure_pi_authenticated(&self.model.canonical()),
            BridgeKind::Codex => crate::codex_sdk::ensure_codex_authenticated(),
        }
    }

    pub async fn ensure_coder_session(&mut self, cwd: &Path) -> Result<(), AgentError> {
        if self.sdk_bridge_needs_restart() {
            self.end_coder_session().await?;
        }
        if self.session.is_some() {
            return Ok(());
        }
        self.begin_coder_session(cwd).await
    }

    #[must_use]
    pub(crate) fn sdk_bridge_needs_restart(&self) -> bool {
        self.session
            .as_ref()
            .is_some_and(|s| s.started_at.elapsed() >= SDK_BRIDGE_MAX_AGE)
    }

    pub async fn begin_coder_session(&mut self, cwd: &Path) -> Result<(), AgentError> {
        reject_no_force(self)?;
        if self.session.is_some() {
            return Err(AgentError(format!(
                "{} SDK session is already open",
                kind_label(self.kind)
            )));
        }
        let cwd = crate::acp::resolve_acp_session_cwd(cwd)?;
        let model = spawn_model_wire(self);
        let thinking = spawn_thinking_wire(self);
        spawn_with_retries(self, cwd, &model, thinking.as_deref()).await
    }

    pub async fn end_coder_session(&mut self) -> Result<(), AgentError> {
        if let Some(s) = self.session.take() {
            if matches!(self.kind, BridgeKind::Cursor) {
                remember_agent_id_from(self, &s);
            }
            s.shutdown().await?;
        }
        Ok(())
    }
}

const fn kind_label(kind: BridgeKind) -> &'static str {
    match kind {
        BridgeKind::Cursor => "cursor",
        BridgeKind::Pi => "pi",
        BridgeKind::Codex => "codex",
    }
}

fn reject_no_force(client: &SdkClient) -> Result<(), AgentError> {
    if client.io.force {
        Ok(())
    } else {
        Err(AgentError(crate::acp::NO_FORCE_MSG.into()))
    }
}

fn spawn_model_wire(client: &SdkClient) -> String {
    match client.kind {
        BridgeKind::Cursor => client.model.cursor_bridge_model(),
        BridgeKind::Pi => client.model.slug.clone(),
        BridgeKind::Codex => client.model.slug.clone(),
    }
}

fn cursor_resume_id(client: &SdkClient) -> Option<String> {
    matches!(client.kind, BridgeKind::Cursor)
        .then(|| client.last_agent_id.clone())
        .flatten()
}

async fn spawn_with_retries(
    client: &mut SdkClient,
    cwd: PathBuf,
    model: &str,
    thinking: Option<&str>,
) -> Result<(), AgentError> {
    let resume_agent_id = cursor_resume_id(client);
    let mut last_error = String::new();
    let max_attempts = client.max_acp_retries;
    let mut attempts_used = 0_u32;
    for attempt in 1..=max_attempts {
        attempts_used = attempt;
        match spawn_for_kind(
            client.kind,
            bridge_spawn_args(client, &cwd, model, thinking),
            resume_agent_id.as_deref(),
            spawn_service_wire(client).as_deref(),
        )
        .await
        {
            Ok(s) => {
                adopt_spawned_session(client, s, cwd);
                return Ok(());
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
        kind_label(client.kind),
        retries_noun(retries)
    )))
}

fn spawn_thinking_wire(client: &SdkClient) -> Option<String> {
    client
        .model
        .thinking_param()
        .filter(|_| matches!(client.kind, BridgeKind::Pi | BridgeKind::Codex))
        .map(str::to_string)
}

fn bridge_spawn_args<'a>(
    client: &'a SdkClient,
    cwd: &'a Path,
    model: &'a str,
    thinking: Option<&'a str>,
) -> BridgeSpawnArgs<'a> {
    BridgeSpawnArgs {
        cwd,
        model,
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
        .filter(|_| matches!(client.kind, BridgeKind::Codex))
        .map(str::to_string)
}

async fn spawn_for_kind(
    kind: BridgeKind,
    args: BridgeSpawnArgs<'_>,
    resume_agent_id: Option<&str>,
    service: Option<&str>,
) -> Result<crate::agent_backend::sdk_session::SdkSession, AgentError> {
    match kind {
        BridgeKind::Cursor => crate::cursor_sdk::spawn_bridge(args, resume_agent_id)
            .await
            .map(|session| SdkSession::Bridge(Box::new(session))),
        BridgeKind::Pi => crate::pi_sdk::spawn_bridge(args).await,
        BridgeKind::Codex => crate::codex_sdk::spawn_bridge(args, service)
            .await
            .map(|session| SdkSession::Bridge(Box::new(session))),
    }
}

fn adopt_spawned_session(
    client: &mut SdkClient,
    s: crate::agent_backend::sdk_session::SdkSession,
    cwd: PathBuf,
) {
    if matches!(client.kind, BridgeKind::Cursor) {
        remember_agent_id_from(client, &s);
    }
    client.session = Some(s);
    client.session_cwd = Some(cwd);
    crate::herdr::notify_reclaim();
}

fn note_spawn_failure(client: &mut SdkClient, err: AgentError) -> String {
    let mut last_error = err.0;
    if matches!(client.kind, BridgeKind::Cursor) && client.last_agent_id.take().is_some() {
        last_error = format!("{last_error} (resume failed; will create)");
    }
    last_error
}

fn remember_agent_id_from(
    client: &mut SdkClient,
    session: &crate::agent_backend::sdk_session::SdkSession,
) {
    let Some(bridge) = session.as_bridge() else {
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
