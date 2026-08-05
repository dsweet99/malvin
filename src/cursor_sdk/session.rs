//! Spawn and talk to the Node bridge process.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex as AsyncMutex;

use crate::acp::{AgentError, AgentIoOptions};

use super::protocol::BridgeRequest;
use super::session_io::{drain_until_run_done, write_request};
use super::session_spawn::spawn_bridge;

pub struct BridgeSession {
    pub child: AsyncMutex<Option<Child>>,
    pub stdin: Arc<AsyncMutex<ChildStdin>>,
    pub stdout: Arc<AsyncMutex<BufReader<ChildStdout>>>,
    pub process_group_id: Option<u32>,
    pub spawn_pid_baseline: HashSet<u32>,
    pub reader_dead: Arc<AtomicBool>,
    pub work_dir: PathBuf,
    pub io: AgentIoOptions,
    pub last_response: Arc<Mutex<String>>,
    pub timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
    pub run_dir: Option<PathBuf>,
}

pub struct BridgeSpawnArgs<'a> {
    pub cwd: &'a Path,
    pub model: &'a str,
    pub io: AgentIoOptions,
    pub run_dir: Option<PathBuf>,
    pub timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
}

impl BridgeSession {
    /// Spawn Node bridge and send `create`.
    pub async fn spawn(args: BridgeSpawnArgs<'_>) -> Result<Self, AgentError> {
        spawn_bridge(args).await
    }

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
        self.reader_dead.store(true, Ordering::SeqCst);
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
