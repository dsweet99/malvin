use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::acp::AgentIoOptions;
use crate::bridge_sdk::BridgeSession;
use crate::model_id::{ModelBackend, ParsedModel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeKind {
    Cursor,
    Pi,
    Codex,
}

pub struct SdkClient {
    pub model: ParsedModel,
    pub kind: BridgeKind,
    pub io: AgentIoOptions,
    pub prompts_log_run_dir: Option<PathBuf>,
    pub max_acp_retries: u32,
    pub(crate) session: Option<BridgeSession>,
    pub(crate) session_cwd: Option<PathBuf>,
    pub(crate) last_agent_id: Option<String>,
    pub(crate) timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
}

struct SdkClientInit {
    model: ParsedModel,
    kind: BridgeKind,
    io: AgentIoOptions,
    max_acp_retries: u32,
}

impl SdkClient {
    #[must_use]
    pub fn new_cursor(model: ParsedModel, io: AgentIoOptions) -> Self {
        Self::from_init(SdkClientInit {
            model,
            kind: BridgeKind::Cursor,
            io,
            max_acp_retries: crate::support_paths::DEFAULT_MAX_ACP_RETRIES,
        })
    }

    #[must_use]
    pub fn new_pi(model: ParsedModel, io: AgentIoOptions) -> Self {
        Self::from_init(SdkClientInit {
            model,
            kind: BridgeKind::Pi,
            io,
            max_acp_retries: crate::support_paths::DEFAULT_MAX_ACP_RETRIES,
        })
    }

    #[must_use]
    pub fn with_max_retries(
        model: ParsedModel,
        kind: BridgeKind,
        io: AgentIoOptions,
        max_acp_retries: u32,
    ) -> Self {
        Self::from_init(SdkClientInit {
            model,
            kind,
            io,
            max_acp_retries,
        })
    }

    #[must_use]
    fn from_init(init: SdkClientInit) -> Self {
        debug_assert!(
            bridge_kind_matches_backend(init.kind, init.model.backend),
            "BridgeKind must match ParsedModel backend"
        );
        Self {
            model: init.model,
            kind: init.kind,
            io: init.io,
            prompts_log_run_dir: None,
            max_acp_retries: if init.max_acp_retries == 0 {
                1
            } else {
                init.max_acp_retries
            },
            session: None,
            session_cwd: None,
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
        self.session.is_some()
    }

    #[must_use]
    pub const fn keeps_coder_session_for_process_life(&self) -> bool {
        true
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

fn sync_timing_to_open_session(client: &mut SdkClient) {
    if let Some(session) = client.session.as_mut() {
        session.timing = client.timing.clone();
    }
}

const fn bridge_kind_matches_backend(kind: BridgeKind, backend: ModelBackend) -> bool {
    matches!(
        (kind, backend),
        (BridgeKind::Cursor, ModelBackend::Cursor)
            | (BridgeKind::Pi, ModelBackend::Pi)
            | (BridgeKind::Codex, ModelBackend::Codex)
    )
}
