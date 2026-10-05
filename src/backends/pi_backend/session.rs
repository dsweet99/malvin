use crate::agent_process::AgentError;
use crate::backends::bridge_sdk::{ChildProcessSession, JsonLineSession, StdioChild, StreamLog};
use crate::config::model_id::ModelBackend;

pub struct NpmPiSession {
    pub stdio: StdioChild,
    pub pi_model: Option<(String, String)>,
    pub local_hold: bool,
    pub output_cap: Option<u64>,
}

impl std::ops::Deref for NpmPiSession {
    type Target = StreamLog;

    fn deref(&self) -> &Self::Target {
        &self.stdio.log
    }
}

impl std::ops::DerefMut for NpmPiSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stdio.log
    }
}

impl ChildProcessSession for NpmPiSession {
    fn stdio(&self) -> &StdioChild {
        &self.stdio
    }

    fn stdio_mut(&mut self) -> &mut StdioChild {
        &mut self.stdio
    }
}

impl JsonLineSession for NpmPiSession {
    const BACKEND: ModelBackend = ModelBackend::Pi;
}

impl NpmPiSession {
    pub async fn send_prompt(&self, prompt: &str) -> Result<(), AgentError> {
        super::session_io::npm_pi_send_prompt(self, prompt).await
    }

    fn release_local_hold(&mut self) {
        if !std::mem::take(&mut self.local_hold) {
            return;
        }
        if let Err(e) = crate::local_llm::release_local_llm() {
            tracing::warn!(target: "malvin::pi_sdk", error = %e, "local llm release failed");
        }
    }

    pub async fn shutdown(mut self) -> Result<(), AgentError> {
        self.release_local_hold();
        let _ = self.write_json(&serde_json::json!({"type":"abort"})).await;
        self.stdio.shutdown_kill_and_clear().await;
        Ok(())
    }
}

impl Drop for NpmPiSession {
    fn drop(&mut self) {
        self.release_local_hold();
    }
}
