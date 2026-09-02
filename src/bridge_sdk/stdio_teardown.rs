//! Shared stdio child teardown for Cursor bridge and Codex sessions.

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::process::Child;
use tokio::sync::Mutex as AsyncMutex;

/// Fields required to tear down a sandboxed stdio agent child.
pub(crate) struct StdioTeardown<'a> {
    pub child: &'a AsyncMutex<Option<Child>>,
    pub process_group_id: Option<u32>,
    pub spawn_pid_baseline: &'a HashSet<u32>,
    pub reader_dead: &'a AtomicBool,
}

impl<'a> StdioTeardown<'a> {
    #[must_use]
    pub(crate) const fn new(
        child: &'a AsyncMutex<Option<Child>>,
        process_group_id: Option<u32>,
        spawn_pid_baseline: &'a HashSet<u32>,
        reader_dead: &'a AtomicBool,
    ) -> Self {
        Self {
            child,
            process_group_id,
            spawn_pid_baseline,
            reader_dead,
        }
    }

    /// Async shutdown path: signal process group, kill child, clear sandbox note.
    pub(crate) async fn shutdown_kill_and_clear(self) {
        self.reader_dead.store(true, Ordering::SeqCst);
        #[cfg(unix)]
        {
            crate::acp::terminate_agent_process_group(
                self.process_group_id,
                self.spawn_pid_baseline,
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
    }

    /// `Drop` path: avoid Tokio destructor on a foreign runtime; always clear sandbox.
    pub(crate) fn drop_teardown(self) {
        self.reader_dead.store(true, Ordering::SeqCst);
        let child_gone = self.child.try_lock().is_ok_and(|slot| slot.is_none());
        if child_gone {
            crate::malvin_sandbox::clear_active_sandbox_session();
            return;
        }
        #[cfg(unix)]
        {
            crate::acp::terminate_agent_process_group_blocking(
                self.process_group_id,
                self.spawn_pid_baseline,
            );
            take_child_without_tokio_drop(self.child);
        }
        #[cfg(not(unix))]
        {
            if let Ok(mut slot) = self.child.try_lock() {
                if let Some(mut child) = slot.take() {
                    let _ = child.start_kill();
                }
            }
        }
        crate::malvin_sandbox::clear_active_sandbox_session();
    }
}

pub(crate) fn drop_stdio_child(
    child: &AsyncMutex<Option<Child>>,
    process_group_id: Option<u32>,
    spawn_pid_baseline: &HashSet<u32>,
    reader_dead: &AtomicBool,
) {
    StdioTeardown::new(child, process_group_id, spawn_pid_baseline, reader_dead).drop_teardown();
}

#[cfg(unix)]
fn take_child_without_tokio_drop(child: &AsyncMutex<Option<Child>>) {
    if tokio::runtime::Handle::try_current().is_ok() {
        return;
    }
    let mut slot = child.blocking_lock();
    if let Some(ch) = slot.take() {
        std::mem::forget(ch);
    }
}
