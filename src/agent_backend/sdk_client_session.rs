
use std::path::{Path, PathBuf};

use crate::acp::{backoff_after_agent_failure, retries_noun, AgentError, AuthError};
use crate::bridge_sdk::{BridgeSpawnArgs, SDK_BRIDGE_MAX_AGE};

use super::sdk_client::{BridgeKind, SdkClient};

impl SdkClient {
    pub fn ensure_authenticated(&self) -> Result<(), AuthError> {
        match self.kind {
            BridgeKind::Cursor => crate::cursor_sdk::ensure_sdk_authenticated(),
            BridgeKind::Pi => crate::pi_sdk::ensure_pi_authenticated(&self.model.canonical()),
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
        if self.session.is_some() {
            return Err(AgentError(format!(
                "{} SDK session is already open",
                kind_label(self.kind)
            )));
        }
        let cwd = crate::acp::resolve_acp_session_cwd(cwd)?;
        let model = spawn_model_wire(self);
        let thinking = spawn_thinking_wire(self);
        let mut last_error = String::new();
        let max_attempts = self.max_acp_retries;
        let mut attempts_used = 0_u32;
        for attempt in 1..=max_attempts {
            attempts_used = attempt;
            match spawn_for_kind(
                self.kind,
                bridge_spawn_args(self, &cwd, &model, thinking.as_deref()),
            )
            .await
            {
                Ok(s) => {
                    adopt_spawned_session(self, s, cwd);
                    return Ok(());
                }
                Err(e) => {
                    last_error = note_spawn_failure(self, e);
                    if backoff_after_agent_failure(
                        self.timing.as_ref(),
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
            kind_label(self.kind),
            retries_noun(retries)
        )))
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
    }
}

fn spawn_model_wire(client: &SdkClient) -> String {
    match client.kind {
        BridgeKind::Cursor => client.model.cursor_bridge_model(),
        BridgeKind::Pi => client.model.slug.clone(),
    }
}

fn spawn_thinking_wire(client: &SdkClient) -> Option<String> {
    client
        .model
        .thinking_param()
        .filter(|_| matches!(client.kind, BridgeKind::Pi))
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
        resume_agent_id: match client.kind {
            BridgeKind::Cursor => client.last_agent_id.clone(),
            BridgeKind::Pi => None,
        },
        normalize_pi_usage: matches!(client.kind, BridgeKind::Pi),
    }
}

async fn spawn_for_kind(
    kind: BridgeKind,
    args: BridgeSpawnArgs<'_>,
) -> Result<crate::bridge_sdk::BridgeSession, AgentError> {
    match kind {
        BridgeKind::Cursor => crate::cursor_sdk::spawn_bridge(args).await,
        BridgeKind::Pi => crate::pi_sdk::spawn_bridge(args).await,
    }
}

fn adopt_spawned_session(
    client: &mut SdkClient,
    s: crate::bridge_sdk::BridgeSession,
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

fn remember_agent_id_from(client: &mut SdkClient, session: &crate::bridge_sdk::BridgeSession) {
    let id = session
        .agent_id
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    if let Some(id) = id {
        client.last_agent_id = Some(id);
    }
}
