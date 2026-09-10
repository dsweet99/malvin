use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex as AsyncMutex;

use crate::acp::AgentError;
use crate::bridge_sdk::{StdioTeardown, StreamLog, drop_stdio_child};

pub struct NpmPiSession {
    pub child: AsyncMutex<Option<Child>>,
    pub stdin: Arc<AsyncMutex<ChildStdin>>,
    pub stdout: Arc<AsyncMutex<BufReader<ChildStdout>>>,
    pub process_group_id: Option<u32>,
    pub spawn_pid_baseline: HashSet<u32>,
    pub reader_dead: Arc<AtomicBool>,
    pub work_dir: PathBuf,
    pub log: StreamLog,
}

impl std::ops::Deref for NpmPiSession {
    type Target = StreamLog;

    fn deref(&self) -> &Self::Target {
        &self.log
    }
}

impl std::ops::DerefMut for NpmPiSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.log
    }
}

impl NpmPiSession {
    pub async fn send_prompt(&self, prompt: &str) -> Result<(), AgentError> {
        super::session_io::npm_pi_send_prompt(self, prompt).await
    }

    pub async fn shutdown(self) -> Result<(), AgentError> {
        let _ = super::session_io::write_json(&self, &serde_json::json!({"type":"abort"})).await;
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
}

impl Drop for NpmPiSession {
    fn drop(&mut self) {
        drop_stdio_child(
            &self.child,
            self.process_group_id,
            &self.spawn_pid_baseline,
            &self.reader_dead,
        );
    }
}
