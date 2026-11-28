use std::sync::Mutex;

use crate::backends::bridge_sdk::{ChildProcessSession, JsonLineSession, StdioChild, StreamLog};
use crate::config::model_id::ModelBackend;

pub struct CodexSession {
    pub stdio: StdioChild,
    pub thread_id: Mutex<Option<String>>,
    pub turn_id: Mutex<Option<String>>,
    pub thread_usage_total: Mutex<Option<serde_json::Value>>,
    pub service: Option<String>,
}

impl std::ops::Deref for CodexSession {
    type Target = StreamLog;

    fn deref(&self) -> &Self::Target {
        &self.stdio.log
    }
}

impl std::ops::DerefMut for CodexSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stdio.log
    }
}

impl ChildProcessSession for CodexSession {
    fn stdio(&self) -> &StdioChild {
        &self.stdio
    }

    fn stdio_mut(&mut self) -> &mut StdioChild {
        &mut self.stdio
    }
}

impl JsonLineSession for CodexSession {
    const BACKEND: ModelBackend = ModelBackend::Codex;
}

pub(crate) fn set_codex_turn_id(session: &CodexSession, turn_id: Option<String>) {
    *session
        .turn_id
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = turn_id;
}
