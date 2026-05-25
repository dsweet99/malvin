use std::collections::VecDeque;
use std::sync::{Arc, Mutex, OnceLock};

use super::sink::DeferredLogSink;
use super::types::DeferredEntry;

pub(crate) type SharedDeferSink = Arc<Mutex<DeferredLogSink>>;

static ACTIVE: OnceLock<Mutex<Option<SharedDeferSink>>> = OnceLock::new();
static PENDING: OnceLock<Mutex<VecDeque<DeferredEntry>>> = OnceLock::new();

fn pending_entries() -> std::sync::MutexGuard<'static, VecDeque<DeferredEntry>> {
    PENDING
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn queue_pending(entry: DeferredEntry) {
    pending_entries().push_back(entry);
}

pub(crate) fn flush_pending_into(sink: &mut DeferredLogSink) {
    let drained: Vec<DeferredEntry> = pending_entries().drain(..).collect();
    for entry in drained {
        sink.push_entry_inner(entry);
    }
}

#[cfg(test)]
pub(crate) fn pending_len() -> usize {
    pending_entries().len()
}

fn active_mutex() -> std::sync::MutexGuard<'static, Option<SharedDeferSink>> {
    ACTIVE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) fn register(sink: SharedDeferSink) {
    super::install_stdout_hooks();
    *active_mutex() = Some(sink);
}

fn spill_orphaned_pending() {
    let pending: Vec<DeferredEntry> = pending_entries().drain(..).collect();
    for entry in &pending {
        super::emit::emit_deferred_entry(entry);
    }
}

pub(crate) fn unregister() {
    let sink = {
        let mut guard = active_mutex();
        guard.take()
    };
    if let Some(sink) = sink {
        if let Ok(mut guard) = sink.try_lock() {
            flush_pending_into(&mut guard);
            pending_entries().clear();
        }
    } else {
        spill_orphaned_pending();
        pending_entries().clear();
    }
}

pub(crate) fn is_registered() -> bool {
    active_mutex().is_some()
}

pub(crate) fn try_log(entry: DeferredEntry) -> bool {
    let sink = {
        let guard = active_mutex();
        guard.as_ref().cloned()
    };
    let Some(sink) = sink else {
        return false;
    };
    let Ok(mut sink_guard) = sink.try_lock() else {
        if let Some(log_line) = crate::output::heartbeat_log_line_for_defer_sink(
            std::time::Instant::now(),
            true,
        ) {
            queue_pending(super::build_heartbeat_entry(log_line));
        }
        queue_pending(entry);
        return true;
    };
    super::log_with_heartbeat(&mut sink_guard, entry);
    true
}

pub(crate) fn try_push(entry: DeferredEntry) -> bool {
    let sink = {
        let guard = active_mutex();
        guard.as_ref().cloned()
    };
    let Some(sink) = sink else {
        return false;
    };
    let Ok(mut sink_guard) = sink.try_lock() else {
        queue_pending(entry);
        return true;
    };
    sink_guard.push_entry(entry);
    true
}

#[cfg(test)]
#[path = "active_tests.rs"]
mod active_tests;

#[cfg(test)]
#[path = "active_tests_pending.rs"]
mod active_tests_pending;
