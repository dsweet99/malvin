use std::path::Path;

use crate::acp::{AgentError, AuthError};
use crate::bridge_sdk::SDK_BRIDGE_MAX_AGE;
use crate::model_id::ModelBackend;

use super::sdk_client::SdkClient;

#[path = "sdk_client_session_spawn.rs"]
mod spawn;

/// Outcome of ensuring a coder session is open.
///
/// Production flows use [`SdkClient::start_coder_session`], which sends the bound
/// header for a fresh agent. [`Fresh`] vs [`Reused`] is recorded for tests and logs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoderSessionEnsure {
    /// A new agent context was created (header is sent by [`SdkClient::start_coder_session`]).
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

    /// Open a coder session and send the bound header when this agent still needs one.
    ///
    /// Call [`Self::bind_session_header`] first. Tests that spawn without prompts may
    /// still use [`Self::begin_coder_session`].
    pub async fn start_coder_session(
        &mut self,
        cwd: &Path,
    ) -> Result<CoderSessionEnsure, AgentError> {
        if self.session_header.is_none() {
            return Err(AgentError(
                "start_coder_session requires bind_session_header so a header is always sent"
                    .into(),
            ));
        }
        let ensure = self.ensure_coder_session(cwd).await?;
        self.deliver_session_header_if_needed().await?;
        Ok(ensure)
    }

    /// Ensure a coder session is open (spawn only; does not send a header).
    ///
    /// Returns [`CoderSessionEnsure::Fresh`] only when a **fresh** agent context was
    /// created. Returns [`CoderSessionEnsure::Reused`] when an open session was
    /// reused, or when a Cursor bridge restart **resumed** a prior `agent_id`.
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

    pub(crate) async fn deliver_session_header_if_needed(&mut self) -> Result<(), AgentError> {
        super::sdk_client_session_header::send_bound_session_header(self).await
    }

    pub async fn end_coder_session(&mut self) -> Result<(), AgentError> {
        let Some(home) = self.coder.as_mut() else {
            return Ok(());
        };
        let Some(s) = home.take_live_session() else {
            return Ok(());
        };
        if matches!(self.model.backend, ModelBackend::Cursor) {
            spawn::remember_agent_id_from(self, &s);
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
async fn begin_coder_session_resumed(
    client: &mut SdkClient,
    cwd: &Path,
) -> Result<bool, AgentError> {
    reject_no_force(client)?;
    if client.has_open_coder_session() {
        return Err(AgentError(format!(
            "{} SDK session is already open",
            client.model.backend.label()
        )));
    }
    let cwd = crate::acp::resolve_acp_session_cwd(cwd)?;
    let thinking = spawn::spawn_thinking_wire(client);
    spawn::spawn_with_retries(client, cwd, thinking.as_deref()).await
}

fn reject_no_force(client: &SdkClient) -> Result<(), AgentError> {
    crate::acp::require_force(client.io.force)
}

#[cfg(test)]
#[path = "sdk_client_session_tests.rs"]
mod sdk_client_session_tests;
