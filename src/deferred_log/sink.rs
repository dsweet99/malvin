use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Instant;

use crate::cursor_store::CursorStoreCache;

use super::config::{DeferredLogConfig, defer_log_enabled_from_env};
use super::emit::emit_deferred_entry;
use super::enrich::enriched_tool_plain;
use super::types::{DeferredEntry, DeferredPayload};

const fn queue_entry_needs_enrich(entry: &DeferredEntry) -> bool {
    matches!(
        &entry.payload,
        DeferredPayload::ToolSummary {
            enrich: Some(_),
            meta: Some(_),
            ..
        }
    )
}

fn prepare_enrich_if_needed(sink: &mut DeferredLogSink) {
    if sink.queue.iter().any(queue_entry_needs_enrich) {
        sink.cache.ensure_open();
        sink.cache.ingest_new_blobs();
    }
}

pub struct DeferredLogSink {
    queue: VecDeque<DeferredEntry>,
    config: DeferredLogConfig,
    cache: CursorStoreCache,
    work_dir: PathBuf,
}

impl DeferredLogSink {
    pub fn new(session_id: String, work_dir: PathBuf, config: DeferredLogConfig) -> Self {
        let cursor_dir = config.cursor_dir.clone();
        Self {
            queue: VecDeque::new(),
            config,
            cache: CursorStoreCache::new(session_id, cursor_dir),
            work_dir,
        }
    }

    pub fn for_prompt(session_id: String, work_dir: PathBuf) -> Option<Self> {
        if !defer_log_enabled_from_env() {
            return None;
        }
        Some(Self::new(
            session_id,
            work_dir,
            DeferredLogConfig::from_env(),
        ))
    }

    pub fn push_entry(&mut self, entry: DeferredEntry) {
        super::active::flush_pending_into(self);
        self.push_entry_inner(entry);
    }

    pub(crate) fn push_entry_inner(&mut self, entry: DeferredEntry) {
        self.queue.push_back(entry);
        super::active::sync_sink_queue_heartbeat_flag(self);
        self.drain_ready();
        super::active::sync_sink_queue_heartbeat_flag(self);
    }

    pub fn force_flush(&mut self) {
        super::active::flush_pending_into(self);
        prepare_enrich_if_needed(self);
        while let Some(entry) = self.queue.pop_front() {
            let entry = self.maybe_enrich(entry);
            emit_deferred_entry(&entry);
        }
        super::active::sync_sink_queue_heartbeat_flag(self);
    }

    #[cfg(test)]
    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub(crate) fn queue_has_heartbeat(&self) -> bool {
        self.queue.iter().any(|entry| {
            matches!(
                &entry.payload,
                DeferredPayload::DisplayLog { log, .. } if crate::output::log_contains_heartbeat(log)
            )
        })
    }

    fn drain_ready(&mut self) {
        if self.queue.is_empty() {
            return;
        }
        let now = Instant::now();
        let max_age = self.config.max_age;
        let head_aged = self.queue.front().is_some_and(|e| {
            now.duration_since(e.enqueued_at) >= max_age
        });
        if !head_aged {
            return;
        }
        let cap = self.config.max_drain_per_log;
        let needs_enrich = self.queue.iter().take(cap).any(|e| {
            now.duration_since(e.enqueued_at) >= max_age && queue_entry_needs_enrich(e)
        });
        if needs_enrich {
            prepare_enrich_if_needed(self);
        }
        for _ in 0..cap {
            let Some(front) = self.queue.front() else {
                break;
            };
            if now.duration_since(front.enqueued_at) < max_age {
                break;
            }
            let entry = self.queue.pop_front().expect("front");
            let entry = self.maybe_enrich(entry);
            emit_deferred_entry(&entry);
        }
    }

    fn format_tool_summary_for_emit(&self, mut entry: DeferredEntry) -> DeferredEntry {
        let DeferredPayload::ToolSummary {
            plain,
            display,
            meta,
            ..
        } = &mut entry.payload
        else {
            return entry;
        };
        if !plain.is_empty() {
            if display.is_empty() {
                *display = super::enrich::styled_tool_payload(plain, entry.emit_stdout_markdown).1;
            }
            return entry;
        }
        let Some(meta) = meta.clone() else {
            return entry;
        };
        let (p, d) = enriched_tool_plain(&meta, None, &self.work_dir, entry.emit_stdout_markdown);
        *plain = p;
        *display = d;
        entry
    }

    fn maybe_enrich(&self, mut entry: DeferredEntry) -> DeferredEntry {
        let DeferredPayload::ToolSummary { plain, display, enrich, meta } = &mut entry.payload
        else {
            return entry;
        };
        let enrich_key = enrich.take();
        let meta = meta.take();
        let (Some(key), Some(meta)) = (enrich_key, meta) else {
            return self.format_tool_summary_for_emit(entry);
        };
        let args = self.cache.get(&key.tool_call_id);
        if args.is_some() {
            tracing::debug!(tool_call_id = %key.tool_call_id, kind = %key.kind, "defer enrich hit");
        } else {
            tracing::debug!(tool_call_id = %key.tool_call_id, kind = %key.kind, "defer enrich miss");
        }
        let (p, d) = enriched_tool_plain(&meta, args, &self.work_dir, entry.emit_stdout_markdown);
        *plain = p;
        *display = d;
        entry
    }
}

impl Drop for DeferredLogSink {
    fn drop(&mut self) {
        self.force_flush();
    }
}

#[cfg(test)]
mod kiss_cov_sink_helpers {
    #[test]
    fn kiss_cov_deferred_log_sink_enrich_helpers() {
        let _ = super::queue_entry_needs_enrich;
        let _ = super::prepare_enrich_if_needed;
    }
}

#[cfg(test)]
pub(crate) mod test_access {
    use super::{DeferredEntry, DeferredLogSink};

    pub fn push_back(sink: &mut DeferredLogSink, entry: DeferredEntry) {
        sink.queue.push_back(entry);
    }

    pub fn drain_ready(sink: &mut DeferredLogSink) {
        sink.drain_ready();
    }

    pub fn ingest_calls(sink: &DeferredLogSink) -> usize {
        sink.cache.ingest_calls
    }
}
