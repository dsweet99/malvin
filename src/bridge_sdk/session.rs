use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex as AsyncMutex;

use crate::acp::AgentError;
use crate::bridge_protocol::BridgeRequest;

use super::session_io::{drain_until_run_done, write_request};
use super::stdio_teardown::{StdioTeardown, drop_stdio_child};
use super::stream_log::StreamLog;

/// Cursor Node JSON-line bridge session.
pub struct BridgeSession {
    pub child: AsyncMutex<Option<Child>>,
    pub stdin: Arc<AsyncMutex<ChildStdin>>,
    pub stdout: Arc<AsyncMutex<BufReader<ChildStdout>>>,
    pub process_group_id: Option<u32>,
    pub spawn_pid_baseline: HashSet<u32>,
    pub reader_dead: Arc<AtomicBool>,
    pub work_dir: PathBuf,
    pub log: StreamLog,
    pub agent_id: Mutex<Option<String>>,
}

impl std::ops::Deref for BridgeSession {
    type Target = StreamLog;

    fn deref(&self) -> &Self::Target {
        &self.log
    }
}

impl std::ops::DerefMut for BridgeSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.log
    }
}

impl BridgeSession {
    pub async fn send_prompt(&self, prompt: &str) -> Result<(), AgentError> {
        let req = BridgeRequest::Send {
            prompt: prompt.to_string(),
            force_stuck: None,
        };
        write_request(self, &req).await?;
        drain_until_run_done(self).await
    }

    pub async fn shutdown(self) -> Result<(), AgentError> {
        let _ = write_request(&self, &BridgeRequest::Cancel {}).await;
        let _ = write_request(&self, &BridgeRequest::Close {}).await;
        StdioTeardown::new(
            &self.child,
            self.process_group_id,
            &self.spawn_pid_baseline,
            &self.reader_dead,
        )
        .shutdown_kill_and_clear()
        .await;
        Ok(())
    }

    fn abandon_child_on_drop(&mut self) {
        drop_stdio_child(
            &self.child,
            self.process_group_id,
            &self.spawn_pid_baseline,
            &self.reader_dead,
        );
    }
}

impl Drop for BridgeSession {
    fn drop(&mut self) {
        self.abandon_child_on_drop();
    }
}
