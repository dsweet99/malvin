use std::sync::Mutex;

use crate::acp::AgentError;
use crate::bridge_sdk::{ChildProcessSession, JsonLineSession, StdioChild, StreamLog};
use crate::model_id::ModelBackend;

pub struct CodexSession {
    pub stdio: StdioChild,
    pub thread_id: Mutex<Option<String>>,
    pub turn_id: Mutex<Option<String>>,
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
    const WIRE_LABEL: &'static str = "codex";
    const PARSE_LABEL: &'static str = "JSON-RPC";
}

impl CodexSession {
    pub async fn send_prompt(&self, prompt: &str) -> Result<(), AgentError> {
        super::session_io::codex_send_prompt(self, prompt).await
    }

    pub async fn shutdown(self) -> Result<(), AgentError> {
        let _ = super::session_io::codex_write_abort(&self).await;
        let _ = super::session_io::codex_delete_thread(&self).await;
        self.stdio.shutdown_kill_and_clear().await;
        Ok(())
    }
}
