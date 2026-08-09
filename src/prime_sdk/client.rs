//! Prime SDK agent client (`AgentBackend::PrimeSdk`).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::acp::AgentIoOptions;

use super::session::PrimeBridgeSession;

pub struct PrimeSdkClient {
    pub model: String,
    pub io: AgentIoOptions,
    pub prompts_log_run_dir: Option<PathBuf>,
    pub max_acp_retries: u32,
    /// Auto-download GGUF for `prime:local/…` (honors `--no-download` when false).
    pub allow_download: bool,
    pub(crate) session: Option<PrimeBridgeSession>,
    /// Resolved cwd from the last successful `begin_coder_session` (kept after teardown for retry).
    pub(crate) session_cwd: Option<PathBuf>,
    pub(crate) timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
}

impl PrimeSdkClient {
    #[must_use]
    pub const fn new(model: String, io: AgentIoOptions) -> Self {
        Self::with_max_retries(model, io, crate::support_paths::DEFAULT_MAX_ACP_RETRIES)
    }

    #[must_use]
    pub const fn with_max_retries(
        model: String,
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
            allow_download: true,
            session: None,
            session_cwd: None,
            timing: None,
        }
    }

    pub fn set_run_timing(
        &mut self,
        timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
    ) {
        self.timing = timing.clone();
        self.prime_sync_session_timing();
    }

    #[must_use]
    pub fn attach_run_timing_for_session(
        &mut self,
    ) -> Arc<Mutex<crate::run_timing::RunTiming>> {
        let timing = crate::run_timing::attach_new_run_timing(&mut self.timing, &self.model);
        self.prime_sync_session_timing();
        timing
    }

    /// Router/`--do` warm-start may `begin_coder_session` before attach; keep session timing in sync.
    fn prime_sync_session_timing(&mut self) {
        if let Some(session) = self.session.as_mut() {
            session.timing = self.timing.clone();
        }
    }

    #[must_use]
    pub const fn has_open_coder_session(&self) -> bool {
        self.session.is_some()
    }

    #[must_use]
    pub fn last_coder_prompt_agent_response(&self) -> Option<String> {
        let session = self.session.as_ref()?;
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
