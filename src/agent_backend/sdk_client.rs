use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::acp::AgentIoOptions;
use crate::model_id::ParsedModel;

use super::sdk_session::SdkSession;

/// Coder session after `begin_coder_session`.
///
/// `NeedsRespawn` keeps cwd so transport teardown can reopen without calling begin again.
pub(crate) enum BegunCoderSession {
    Live { cwd: PathBuf, session: SdkSession },
    NeedsRespawn { cwd: PathBuf },
}

impl BegunCoderSession {
    #[must_use]
    pub(crate) const fn cwd(&self) -> &PathBuf {
        match self {
            Self::Live { cwd, .. } | Self::NeedsRespawn { cwd } => cwd,
        }
    }

    #[must_use]
    pub(crate) const fn live_session(&self) -> Option<&SdkSession> {
        match self {
            Self::Live { session, .. } => Some(session),
            Self::NeedsRespawn { .. } => None,
        }
    }

    #[must_use]
    pub(crate) const fn live_session_mut(&mut self) -> Option<&mut SdkSession> {
        match self {
            Self::Live { session, .. } => Some(session),
            Self::NeedsRespawn { .. } => None,
        }
    }

    /// Take the live session, leaving [`NeedsRespawn`] with the same cwd.
    pub(crate) fn take_live_session(&mut self) -> Option<SdkSession> {
        match std::mem::replace(
            self,
            Self::NeedsRespawn {
                cwd: PathBuf::new(),
            },
        ) {
            Self::Live { cwd, session } => {
                *self = Self::NeedsRespawn { cwd };
                Some(session)
            }
            Self::NeedsRespawn { cwd } => {
                *self = Self::NeedsRespawn { cwd };
                None
            }
        }
    }
}

pub struct SdkClient {
    pub model: ParsedModel,
    pub io: AgentIoOptions,
    pub prompts_log_run_dir: Option<PathBuf>,
    pub max_acp_retries: u32,
    pub(crate) coder: Option<BegunCoderSession>,
    pub(crate) last_agent_id: Option<String>,
    pub(crate) timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
}

impl SdkClient {
    #[must_use]
    pub const fn new(model: ParsedModel, io: AgentIoOptions) -> Self {
        Self::with_max_retries(model, io, crate::support_paths::DEFAULT_MAX_ACP_RETRIES)
    }

    #[must_use]
    pub const fn with_max_retries(
        model: ParsedModel,
        io: AgentIoOptions,
        max_acp_retries: u32,
    ) -> Self {
        Self {
            model,
            io,
            prompts_log_run_dir: None,
            max_acp_retries: if max_acp_retries == 0 {
                1
            } else {
                max_acp_retries
            },
            coder: None,
            last_agent_id: None,
            timing: None,
        }
    }

    pub fn set_run_timing(&mut self, timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>) {
        self.timing = timing.clone();
        sync_timing_to_open_session(self);
    }

    #[must_use]
    pub fn attach_run_timing_for_session(&mut self) -> Arc<Mutex<crate::run_timing::RunTiming>> {
        let model = self.model.canonical();
        let timing = crate::run_timing::attach_new_run_timing(&mut self.timing, &model);
        sync_timing_to_open_session(self);
        timing
    }

    #[must_use]
    pub const fn has_open_coder_session(&self) -> bool {
        matches!(self.coder, Some(BegunCoderSession::Live { .. }))
    }

    #[must_use]
    pub fn last_coder_prompt_agent_response(&self) -> Option<String> {
        let session = live_session(self)?;
        let text = session
            .last_response
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        if text.is_empty() || text == "\0" {
            None
        } else {
            Some(text)
        }
    }
}

#[cfg(test)]
#[must_use]
pub const fn new_cursor(model: ParsedModel, io: AgentIoOptions) -> SdkClient {
    SdkClient::new(model, io)
}

#[cfg(test)]
#[must_use]
pub const fn new_pi(model: ParsedModel, io: AgentIoOptions) -> SdkClient {
    SdkClient::new(model, io)
}

#[cfg(test)]
#[must_use]
pub const fn new_codex(model: ParsedModel, io: AgentIoOptions) -> SdkClient {
    SdkClient::new(model, io)
}

#[must_use]
pub(crate) fn live_session(client: &SdkClient) -> Option<&SdkSession> {
    client
        .coder
        .as_ref()
        .and_then(BegunCoderSession::live_session)
}

#[must_use]
pub(crate) fn live_session_mut(client: &mut SdkClient) -> Option<&mut SdkSession> {
    client
        .coder
        .as_mut()
        .and_then(BegunCoderSession::live_session_mut)
}

#[must_use]
pub(crate) fn begun_cwd(client: &SdkClient) -> Option<&PathBuf> {
    client.coder.as_ref().map(BegunCoderSession::cwd)
}

fn sync_timing_to_open_session(client: &mut SdkClient) {
    let timing = client.timing.clone();
    if let Some(session) = live_session_mut(client) {
        session.timing = timing;
    }
}
