use std::sync::Mutex;

use crate::agent_process::AgentError;
use crate::backends::cursor_sdk::protocol::BridgeRequest;

use super::child_process_session::ChildProcessSession;
use super::session_io::{drain_until_run_done, write_request};
use super::stdio_child::StdioChild;
use super::stream_log::StreamLog;

pub struct BridgeSession {
    pub stdio: StdioChild,
    pub agent_id: Mutex<Option<String>>,
}

impl std::ops::Deref for BridgeSession {
    type Target = StreamLog;

    fn deref(&self) -> &Self::Target {
        &self.stdio.log
    }
}

impl std::ops::DerefMut for BridgeSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stdio.log
    }
}

impl ChildProcessSession for BridgeSession {
    fn stdio(&self) -> &StdioChild {
        &self.stdio
    }

    fn stdio_mut(&mut self) -> &mut StdioChild {
        &mut self.stdio
    }
}

impl BridgeSession {
    pub async fn send_prompt(&self, prompt: &str) -> Result<(), AgentError> {
        let req = BridgeRequest::Send {
            prompt: prompt.to_string(),
        };
        write_request(self, &req).await?;
        drain_until_run_done(self).await
    }

    pub async fn shutdown(self) -> Result<(), AgentError> {
        let _ = write_request(&self, &BridgeRequest::Cancel {}).await;
        let _ = write_request(&self, &BridgeRequest::Close {}).await;
        self.stdio.shutdown_kill_and_clear().await;
        Ok(())
    }
}
