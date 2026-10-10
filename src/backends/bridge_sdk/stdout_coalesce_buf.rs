use std::sync::{Arc, Condvar, Mutex, MutexGuard, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

pub(crate) use super::bridge_sdk_stdout_coalesce_buf_worker as worker;

use crate::agent_process::{
    AgentIoOptions, SessionUpdateChunkKind, TraceChunkCoalescer, TraceChunkEmission,
};

pub(crate) const STDOUT_COALESCE_IDLE: Duration = Duration::from_millis(250);

pub(crate) struct CoalesceSlot {
    pub(crate) inner: TraceChunkCoalescer,
    pub(crate) deadline: Option<Instant>,
    pub(crate) stop: bool,
}

pub(crate) struct Shared {
    pub(crate) slot: Mutex<CoalesceSlot>,
    pub(crate) cv: Condvar,
    pub(crate) idle: Duration,
    pub(crate) io: AgentIoOptions,
    #[cfg(test)]
    pub(crate) test_lines: Mutex<Vec<String>>,
}

pub(crate) struct StdoutCoalesceBuf {
    shared: Arc<Shared>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl StdoutCoalesceBuf {
    #[must_use]
    pub(crate) fn new(io: AgentIoOptions) -> Self {
        build(io, STDOUT_COALESCE_IDLE)
    }

    pub(crate) fn feed_and_write(&self, kind: SessionUpdateChunkKind, chunk: &str) {
        let emissions = feed(self, kind, chunk);
        write_coalesced_emissions(self.shared.io, emissions);
    }

    pub(crate) fn flush_and_write(&self) {
        let emissions = flush(self);
        write_coalesced_emissions(self.shared.io, emissions);
    }
}

impl Drop for StdoutCoalesceBuf {
    fn drop(&mut self) {
        request_stop(&self.shared);
        let handle = self
            .worker
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(handle) = handle {
            let _ = handle.join();
        }
    }
}

fn build(io: AgentIoOptions, idle: Duration) -> StdoutCoalesceBuf {
    StdoutCoalesceBuf {
        shared: Arc::new(Shared {
            slot: Mutex::new(CoalesceSlot {
                inner: TraceChunkCoalescer::default(),
                deadline: None,
                stop: false,
            }),
            cv: Condvar::new(),
            idle,
            io,
            #[cfg(test)]
            test_lines: Mutex::new(Vec::new()),
        }),
        worker: Mutex::new(None),
    }
}

fn feed(
    buf: &StdoutCoalesceBuf,
    kind: SessionUpdateChunkKind,
    chunk: &str,
) -> Vec<TraceChunkEmission> {
    let mut slot = lock_slot(&buf.shared);
    let emissions = slot.inner.feed(kind, chunk);
    arm(&buf.shared, &mut slot);
    drop(slot);
    wake_worker(buf);
    emissions
}

fn flush(buf: &StdoutCoalesceBuf) -> Vec<TraceChunkEmission> {
    let mut slot = lock_slot(&buf.shared);
    let emissions = slot.inner.flush_all();
    arm(&buf.shared, &mut slot);
    drop(slot);
    emissions
}

fn request_stop(shared: &Shared) {
    let mut slot = lock_slot(shared);
    slot.stop = true;
    notify(shared, &slot);
    drop(slot);
}

fn wake_worker(buf: &StdoutCoalesceBuf) {
    let slot = lock_slot(&buf.shared);
    let armed = slot.deadline.is_some();
    drop(slot);
    if !armed {
        return;
    }
    ensure_worker(buf);
    let slot = lock_slot(&buf.shared);
    notify(&buf.shared, &slot);
    drop(slot);
}

fn ensure_worker(buf: &StdoutCoalesceBuf) {
    let mut guard = buf.worker.lock().unwrap_or_else(PoisonError::into_inner);
    if guard.is_some() {
        return;
    }
    let shared = Arc::clone(&buf.shared);
    *guard = Some(thread::spawn(move || worker::worker_loop(shared)));
}

fn arm(shared: &Shared, slot: &mut CoalesceSlot) {
    slot.deadline = None;
    if slot.inner.has_pending() {
        slot.deadline = Some(Instant::now() + shared.idle);
    }
    notify(shared, slot);
}

fn notify(shared: &Shared, slot: &CoalesceSlot) {
    let _ = slot.deadline;
    shared.cv.notify_one();
}

pub(crate) fn lock_slot(shared: &Shared) -> MutexGuard<'_, CoalesceSlot> {
    shared.slot.lock().unwrap_or_else(PoisonError::into_inner)
}

pub(crate) fn write_coalesced_emissions(io: AgentIoOptions, emissions: Vec<TraceChunkEmission>) {
    if io.no_tee {
        return;
    }
    for (kind, line, ..) in emissions {
        write_one(io, kind, &line);
    }
}

fn write_one(io: AgentIoOptions, kind: SessionUpdateChunkKind, line: &str) {
    if skip_line(io, kind, line) {
        return;
    }
    let who = who_for(kind);
    if io.raw_output && kind == SessionUpdateChunkKind::Message {
        crate::output::print_stdout_text_with_markdown(who, line, io.emit_stdout_markdown);
        return;
    }
    crate::output::print_stdout_line_with_markdown(who, line, io.emit_stdout_markdown);
}

fn skip_line(io: AgentIoOptions, kind: SessionUpdateChunkKind, line: &str) -> bool {
    line.is_empty() || (kind == SessionUpdateChunkKind::Thought && !io.show_thoughts_on_stdout)
}

const fn who_for(kind: SessionUpdateChunkKind) -> &'static str {
    match kind {
        SessionUpdateChunkKind::Message => crate::output::WHO_M,
        SessionUpdateChunkKind::Thought => crate::output::WHO_B,
    }
}

#[cfg(test)]
fn pending(buf: &StdoutCoalesceBuf) -> bool {
    lock_slot(&buf.shared).inner.has_pending()
}

#[cfg(test)]
fn recorded(buf: &StdoutCoalesceBuf) -> Vec<String> {
    buf.shared
        .test_lines
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

#[cfg(test)]
fn idle_of(buf: &StdoutCoalesceBuf) -> Duration {
    buf.shared.idle
}

#[cfg(test)]
#[path = "stdout_coalesce_buf_tests.rs"]
mod tests;
