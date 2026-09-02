use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex as AsyncMutex;

use crate::acp::AgentError;
use crate::bridge_sdk::{StreamLog, StdioTeardown, drop_stdio_child};

/// Codex app-server JSON-RPC session (stdio child process).
pub struct CodexSession {
    pub child: AsyncMutex<Option<Child>>,
    pub stdin: Arc<AsyncMutex<ChildStdin>>,
    pub stdout: Arc<AsyncMutex<BufReader<ChildStdout>>>,
    pub process_group_id: Option<u32>,
    pub spawn_pid_baseline: HashSet<u32>,
    pub reader_dead: Arc<AtomicBool>,
    pub work_dir: PathBuf,
    pub log: StreamLog,
    pub thread_id: Mutex<Option<String>>,
    pub turn_id: Mutex<Option<String>>,
    pub service: Option<String>,
}

impl std::ops::Deref for CodexSession {
    type Target = StreamLog;

    fn deref(&self) -> &Self::Target {
        &self.log
    }
}

impl std::ops::DerefMut for CodexSession {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.log
    }
}

impl CodexSession {
    pub async fn send_prompt(&self, prompt: &str) -> Result<(), AgentError> {
        super::session_io::codex_send_prompt(self, prompt).await
    }

    pub async fn shutdown(self) -> Result<(), AgentError> {
        let _ = super::session_io::codex_write_abort(&self).await;
        let _ = super::session_io::codex_delete_thread(&self).await;
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

    fn clear_orphaned_stdio(session: &mut Self) {
        drop_stdio_child(
            &session.child,
            session.process_group_id,
            &session.spawn_pid_baseline,
            &session.reader_dead,
        );
    }
}

impl Drop for CodexSession {
    fn drop(&mut self) {
        Self::clear_orphaned_stdio(self);
    }
}
