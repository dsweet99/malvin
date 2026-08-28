use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex as AsyncMutex;

use crate::acp::AgentError;
use crate::bridge_sdk::StreamLog;

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
        self.reader_dead.store(true, Ordering::SeqCst);
        let _ = super::session_io::codex_write_abort(&self).await;
        let _ = super::session_io::codex_delete_thread(&self).await;
        #[cfg(unix)]
        {
            crate::acp::terminate_agent_process_group(
                self.process_group_id,
                &self.spawn_pid_baseline,
            )
            .await;
        }
        {
            let mut child_slot = self.child.lock().await;
            if let Some(mut child) = child_slot.take() {
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
        }
        crate::malvin_sandbox::clear_active_sandbox_session();
        Ok(())
    }
}

impl Drop for CodexSession {
    fn drop(&mut self) {
        codex_session_drop_teardown(self);
    }
}

fn codex_session_drop_teardown(session: &CodexSession) {
    session.reader_dead.store(true, Ordering::SeqCst);
    let child_gone = session.child.try_lock().is_ok_and(|slot| slot.is_none());
    if child_gone {
        crate::malvin_sandbox::clear_active_sandbox_session();
        return;
    }
    #[cfg(unix)]
    {
        crate::acp::terminate_agent_process_group_blocking(
            session.process_group_id,
            &session.spawn_pid_baseline,
        );
        take_codex_child_without_tokio_drop(session);
    }
    #[cfg(not(unix))]
    {
        if let Ok(mut slot) = session.child.try_lock() {
            if let Some(mut child) = slot.take() {
                let _ = child.start_kill();
            }
        }
    }
    crate::malvin_sandbox::clear_active_sandbox_session();
}

#[cfg(unix)]
fn take_codex_child_without_tokio_drop(session: &CodexSession) {
    if tokio::runtime::Handle::try_current().is_ok() {
        return;
    }
    let mut slot = session.child.blocking_lock();
    if let Some(ch) = slot.take() {
        std::mem::forget(ch);
    }
}
