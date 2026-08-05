//! Cursor SDK agent client (`AgentBackend::CursorSdk`).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::acp::AgentIoOptions;

use super::session::BridgeSession;

pub struct CursorSdkClient {
    pub model: String,
    pub io: AgentIoOptions,
    pub prompts_log_run_dir: Option<PathBuf>,
    pub max_acp_retries: u32,
    pub(crate) session: Option<BridgeSession>,
    pub(crate) timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
}

impl CursorSdkClient {
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
            session: None,
            timing: None,
        }
    }

    pub fn set_run_timing(
        &mut self,
        timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
    ) {
        self.timing = timing;
    }

    #[must_use]
    pub fn attach_run_timing_for_session(
        &mut self,
    ) -> Arc<Mutex<crate::run_timing::RunTiming>> {
        crate::run_timing::attach_new_run_timing(&mut self.timing)
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
        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }
}
