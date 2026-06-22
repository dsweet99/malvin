use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use super::sink::DeferredLogSink;
use super::types::{DeferredEntry, DeferredPayload};

pub(crate) type SharedDeferSink = Arc<Mutex<DeferredLogSink>>;

static ACTIVE: OnceLock<Mutex<Option<SharedDeferSink>>> = OnceLock::new();
static PENDING: OnceLock<Mutex<VecDeque<DeferredEntry>>> = OnceLock::new();
static SINK_QUEUE_HAS_HEARTBEAT: AtomicBool = AtomicBool::new(false);

fn pending_entries() -> std::sync::MutexGuard<'static, VecDeque<DeferredEntry>> {
    PENDING
        .get_or_init(|| Mutex::new(VecDeque::new()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn queue_pending(entry: DeferredEntry) {
    pending_entries().push_back(entry);
}

fn entry_is_heartbeat(entry: &DeferredEntry) -> bool {
    matches!(
        &entry.payload,
        DeferredPayload::DisplayLog { log, .. } if crate::output::log_contains_heartbeat(log)
    )
}

fn pending_has_heartbeat() -> bool {
    pending_entries().iter().any(entry_is_heartbeat)
}

pub(crate) fn sync_sink_queue_heartbeat_flag(sink: &DeferredLogSink) {
    SINK_QUEUE_HAS_HEARTBEAT.store(sink.queue_has_heartbeat(), Ordering::Relaxed);
}

fn heartbeat_already_deferred(sink: &SharedDeferSink) -> bool {
    if pending_has_heartbeat() {
        return true;
    }
    let cached = SINK_QUEUE_HAS_HEARTBEAT.load(Ordering::Relaxed);
    sink.try_lock().map_or(cached, |guard| {
        let has = guard.queue_has_heartbeat();
        SINK_QUEUE_HAS_HEARTBEAT.store(has, Ordering::Relaxed);
        has
    })
}

pub(crate) fn defer_already_has_heartbeat(sink: &DeferredLogSink) -> bool {
    pending_has_heartbeat() || sink.queue_has_heartbeat()
}

pub(crate) fn heartbeat_live_pending() -> bool {
    pending_has_heartbeat() || SINK_QUEUE_HAS_HEARTBEAT.load(Ordering::Relaxed)
}

pub(crate) fn flush_pending_into(sink: &mut DeferredLogSink) {
    let drained: Vec<DeferredEntry> = pending_entries().drain(..).collect();
    for entry in drained {
        sink.push_entry_inner(entry);
    }
}

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
        match sink.try_lock() {
            Ok(mut guard) => {
                flush_pending_into(&mut guard);
                guard.force_flush();
            }
            Err(_) => spill_orphaned_pending(),
        }
        pending_entries().clear();
    } else {
        spill_orphaned_pending();
        pending_entries().clear();
        SINK_QUEUE_HAS_HEARTBEAT.store(false, Ordering::Relaxed);
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
        if !heartbeat_already_deferred(&sink) {
            if let Some((display, log)) =
                crate::output::heartbeat_rendered_if_due(std::time::Instant::now(), true)
            {
                crate::output::publish_heartbeat_live_terminal(&display);
                queue_pending(super::build_display_log_entry(display, log));
            }
        }
        queue_pending(entry);
        return true;
    };
    super::log_with_heartbeat(&mut sink_guard, entry);
    true
}

pub(crate) fn defer_sink_mutex_held() -> bool {
    let sink = {
        let guard = active_mutex();
        guard.as_ref().cloned()
    };
    let Some(sink) = sink else {
        return false;
    };
    sink.try_lock().is_err()
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
        if entry_is_heartbeat(&entry) {
            if let DeferredPayload::DisplayLog { display, .. } = &entry.payload {
                if !heartbeat_already_deferred(&sink) {
                    crate::output::publish_heartbeat_live_terminal(display);
                }
            }
        }
        if !(entry_is_heartbeat(&entry) && heartbeat_already_deferred(&sink)) {
            queue_pending(entry);
        }
        return true;
    };
    if entry_is_heartbeat(&entry) && defer_already_has_heartbeat(&sink_guard) {
        return true;
    }
    sink_guard.push_entry(entry);
    true
}

#[cfg(test)]
#[path = "active_tests_inline.rs"]
mod active_tests_inline;
#[cfg(test)]
#[path = "active_tests_pending.rs"]
mod active_tests_pending;

#[cfg(test)]
#[path = "active_tests_sink_queue.rs"]
mod active_tests_sink_queue;

#[cfg(test)]
#[path = "active_tests_contention_bugs.rs"]
mod active_tests_contention_bugs;
#[cfg(test)]
#[path = "active_test.rs"]
mod active_test;
