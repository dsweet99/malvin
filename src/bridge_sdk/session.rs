use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex as AsyncMutex;

use crate::acp::{AgentError, AgentIoOptions};
use crate::bridge_protocol::BridgeRequest;

use super::session_io::{drain_until_run_done, write_request};
use super::stream_log::StreamLog;

pub const SDK_BRIDGE_MAX_AGE: Duration = Duration::from_mins(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeWire {
    NodeBridge,
    CodexRpc,
}

#[derive(Debug, Clone)]
pub struct ToolCallStart {
    pub started: Instant,
    pub summary: String,
}

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
    pub turn_id: Mutex<Option<String>>,
    pub service: Option<String>,
    pub wire: BridgeWire,
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

pub struct BridgeSpawnArgs<'a> {
    pub cwd: &'a Path,
    pub model: &'a str,
    pub thinking: Option<&'a str>,
    pub io: AgentIoOptions,
    pub run_dir: Option<PathBuf>,
    pub timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
}

impl BridgeSession {
    pub async fn send_prompt(&self, prompt: &str) -> Result<(), AgentError> {
        match self.wire {
            BridgeWire::NodeBridge => {
                let req = BridgeRequest::Send {
                    prompt: prompt.to_string(),
                    force_stuck: None,
                };
                write_request(self, &req).await?;
                drain_until_run_done(self).await
            }
            BridgeWire::CodexRpc => crate::codex_sdk::send_prompt(self, prompt).await,
        }
    }

    pub async fn shutdown(self) -> Result<(), AgentError> {
        self.reader_dead.store(true, Ordering::SeqCst);
        match self.wire {
            BridgeWire::NodeBridge => {
                let _ = write_request(&self, &BridgeRequest::Cancel {}).await;
                let _ = write_request(&self, &BridgeRequest::Close {}).await;
            }
            BridgeWire::CodexRpc => {
                let _ = crate::codex_sdk::write_abort(&self).await;
                let _ = crate::codex_sdk::delete_thread(&self).await;
            }
        }
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

impl Drop for BridgeSession {
    fn drop(&mut self) {
        bridge_session_drop_teardown(self);
    }
}

fn bridge_session_drop_teardown(session: &BridgeSession) {
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
        take_bridge_child_without_tokio_drop(session);
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
fn take_bridge_child_without_tokio_drop(session: &BridgeSession) {
    if tokio::runtime::Handle::try_current().is_ok() {
        return;
    }
    let mut slot = session.child.blocking_lock();
    if let Some(ch) = slot.take() {
        std::mem::forget(ch);
    }
}
