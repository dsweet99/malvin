//! Audit trace events (`miniTerminal`, shrink, fork) for [`super::trace::MiniTraceSink`].
//!
//! All functions here are audit-only by construction (see [`crate::observability`]).
use super::acp_trace_shim::{
    emit_mini_prompt_shrink, emit_mini_prompt_shrink_stalled, emit_mini_retry_fork,
    emit_mini_terminal,
};
use super::context_recovery::{DROP_STRATEGY_OLDEST_WHOLE, ShrinkEvent};
use super::retry_fork::RetryForkLedger;
use super::terminal::MiniTerminalRecord;
use super::trace::{emit_audit, MiniTraceSink};
use crate::observability::AuditEventKind;

pub(crate) fn emit_terminal(sink: &MiniTraceSink, record: &MiniTerminalRecord) {
    emit_audit(sink, AuditEventKind::MiniTerminal, |trace| {
        emit_mini_terminal(trace, record);
    });
}

pub(crate) fn emit_prompt_shrink(sink: &MiniTraceSink, event: &ShrinkEvent) {
    emit_audit(sink, AuditEventKind::MiniPromptShrink, |trace| {
        emit_mini_prompt_shrink(
            trace,
            super::acp_trace_shim::MiniPromptShrinkTrace {
                attempt: event.attempt,
                messages_before: event.messages_before,
                messages_after: event.messages_after,
                bytes_removed: event.bytes_removed,
                strategy: DROP_STRATEGY_OLDEST_WHOLE,
            },
        );
    });
}

pub(crate) fn emit_prompt_shrink_stalled(sink: &MiniTraceSink) {
    emit_audit(sink, AuditEventKind::MiniPromptShrinkStalled, |trace| {
        emit_mini_prompt_shrink_stalled(trace);
    });
}

pub(crate) fn emit_retry_fork(sink: &MiniTraceSink, ledger: &RetryForkLedger) {
    emit_audit(sink, AuditEventKind::MiniRetryFork, |trace| {
        emit_mini_retry_fork(trace, ledger);
    });
}
