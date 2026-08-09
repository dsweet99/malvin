//! Spawn and talk to the Node bridge process.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tokio::io::BufReader;
use tokio::process::{Child, ChildStdin, ChildStdout};
use tokio::sync::Mutex as AsyncMutex;

use crate::acp::{AgentError, AgentIoOptions};

use super::protocol::PrimeBridgeRequest;
use super::session_io::{prime_drain_until_run_done, prime_write_request};
use super::session_spawn::prime_spawn_bridge;

/// Cursor-side long-lived SDK connections time out (~1.5h). When starting an
/// agent, restart the Node bridge if it has been alive at least this long.
pub(crate) const SDK_BRIDGE_MAX_AGE: Duration = Duration::from_secs(10 * 60);

/// Cached tool-call start for ACP-parity done-line timing.
#[derive(Debug, Clone)]
pub struct PrimeToolCallStart {
    pub started: Instant,
    pub summary: String,
}

pub struct PrimeBridgeSession {
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
    /// When this bridge process was spawned (`Instant::now` at assemble).
    pub started_at: Instant,
    /// Session id from create `ok` (informational; no resume in v1).
    pub agent_id: Mutex<Option<String>>,
    /// ACP-parity stdout coalescer for streamed assistant/thinking chunks.
    pub stdout_coalesce: Mutex<crate::acp::TraceChunkCoalescer>,
    /// toolCallId → start instant + summary (for done-line duration).
    pub tool_starts: Mutex<HashMap<String, PrimeToolCallStart>>,
    /// Keeps the GGUF OpenAI-compatible sidecar + temp `models.json` alive for the session.
    pub local_sidecar: Option<crate::local_llm::PrimeLocalSidecar>,
}

pub struct PrimeBridgeSpawnArgs<'a> {
    pub cwd: &'a Path,
    pub model: &'a str,
    pub io: AgentIoOptions,
    pub run_dir: Option<PathBuf>,
    pub timing: Option<Arc<Mutex<crate::run_timing::RunTiming>>>,
    /// When set, start malvin local GGUF sidecar before `create`.
    pub allow_download: bool,
    pub prime_local: bool,
}

impl PrimeBridgeSession {
    /// Spawn Node bridge and send `create`.
    pub async fn spawn(args: PrimeBridgeSpawnArgs<'_>) -> Result<Self, AgentError> {
        prime_spawn_bridge(args).await
    }

    pub async fn send_prompt(&self, prompt: &str) -> Result<(), AgentError> {
        let req = PrimeBridgeRequest::Send {
            prompt: prompt.to_string(),
        };
        prime_write_request(self, &req).await?;
        prime_drain_until_run_done(self).await
    }

    pub async fn shutdown(self) -> Result<(), AgentError> {
        self.reader_dead.store(true, Ordering::SeqCst);
        // Best-effort cancel/close, then full sandbox PG teardown (parity with Cursor SDK /
        // ACP). PID-only kill left tool children alive and caused Drop to skip PG teardown.
        let _ = prime_write_request(&self, &PrimeBridgeRequest::Cancel {}).await;
        let _ = prime_write_request(&self, &PrimeBridgeRequest::Close {}).await;
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

impl Drop for PrimeBridgeSession {
    fn drop(&mut self) {
        prime_bridge_session_drop_teardown(self);
    }
}

fn prime_bridge_session_drop_teardown(session: &PrimeBridgeSession) {
    session.reader_dead.store(true, Ordering::SeqCst);
    let child_gone = session
        .child
        .try_lock()
        .map(|slot| slot.is_none())
        .unwrap_or(false);
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
        prime_take_bridge_child_without_tokio_drop(session);
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
fn prime_take_bridge_child_without_tokio_drop(session: &PrimeBridgeSession) {
    if tokio::runtime::Handle::try_current().is_ok() {
        return;
    }
    let mut slot = session.child.blocking_lock();
    if let Some(ch) = slot.take() {
        std::mem::forget(ch);
    }
}
