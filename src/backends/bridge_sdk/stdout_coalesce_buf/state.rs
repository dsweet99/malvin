use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::time::{Duration, Instant};

use crate::agent_process::{
    AgentIoOptions, SessionUpdateChunkKind, TraceChunkCoalescer, TraceChunkEmission,
};

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

pub(crate) fn mark_line_open(io: AgentIoOptions, open: bool) {
    if !io.no_tee {
        crate::output::set_agent_text_line_open(open);
    }
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
