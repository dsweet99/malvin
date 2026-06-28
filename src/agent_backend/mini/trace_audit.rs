//! Audit trace events (`miniTerminal`, shrink, fork) for [`super::trace::MiniTraceSink`].

use super::acp_trace_shim::{
    emit_mini_prompt_shrink, emit_mini_prompt_shrink_stalled, emit_mini_retry_fork,
    emit_mini_terminal,
};
use super::context_recovery::{DROP_STRATEGY_OLDEST_WHOLE, ShrinkEvent};
use super::retry_fork::RetryForkLedger;
use super::terminal::MiniTerminalRecord;
use super::trace::MiniTraceSink;

pub(crate) fn emit_terminal(sink: &MiniTraceSink, record: &MiniTerminalRecord) {
    if let Some(trace) = sink.acp_trace_for_audit() {
        emit_mini_terminal(&trace, record);
    }
}

pub(crate) fn emit_prompt_shrink(sink: &MiniTraceSink, event: &ShrinkEvent) {
    if let Some(trace) = sink.acp_trace_for_audit() {
        emit_mini_prompt_shrink(
            &trace,
            super::acp_trace_shim::MiniPromptShrinkTrace {
                attempt: event.attempt,
                messages_before: event.messages_before,
                messages_after: event.messages_after,
                bytes_removed: event.bytes_removed,
                strategy: DROP_STRATEGY_OLDEST_WHOLE,
            },
        );
    }
}

pub(crate) fn emit_prompt_shrink_stalled(sink: &MiniTraceSink) {
    if let Some(trace) = sink.acp_trace_for_audit() {
        emit_mini_prompt_shrink_stalled(&trace);
    }
}

pub(crate) fn emit_retry_fork(sink: &MiniTraceSink, ledger: &RetryForkLedger) {
    if let Some(trace) = sink.acp_trace_for_audit() {
        emit_mini_retry_fork(&trace, ledger);
    }
}
