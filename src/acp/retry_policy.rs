use crate::acp::import_prelude::*;
// Bounded retries for transient ACP JSON-RPC failures (see product plan: up to 3 attempts, 1s / 3s backoff).
// Covers `session/prompt` and spawn/handshake (`initialize` / `authenticate` / `session/new`) via
// [`AgentClient::begin_coder_session`], which retries the full `agent acp` session setup.

pub(crate) const MAX_AGENT_ATTEMPTS: u32 = 3;

/// English noun for `n` retry attempts after the first try (`n` is `attempts_used - 1` in callers).
#[must_use]
pub(crate) const fn retries_noun(n: u32) -> &'static str {
    if n == 1 { "retry" } else { "retries" }
}

pub(crate) fn agent_string_is_upgrade_plan(msg: &str) -> bool {
    msg.to_ascii_lowercase()
        .contains("upgrade your plan to continue")
}

pub(crate) fn agent_string_is_cannot_use_model(msg: &str) -> bool {
    msg.to_ascii_lowercase().contains("cannot use this model")
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum IterableClosedStream {
    Writable,
    Readable,
}

/// Which iterable-closed error the coalesced buffer carries, if any.
#[must_use]
pub(crate) fn iterable_closed_stream_from_buffer(buf: &str) -> Option<IterableClosedStream> {
    let text = buf.to_ascii_lowercase();
    if text.contains("readableiterable is closed") {
        Some(IterableClosedStream::Readable)
    } else if text.contains("writableiterable is closed") {
        Some(IterableClosedStream::Writable)
    } else {
        None
    }
}

const fn iterable_closed_stream_message(kind: IterableClosedStream) -> &'static str {
    match kind {
        IterableClosedStream::Writable => "acp: WritableIterable is closed",
        IterableClosedStream::Readable => "acp: ReadableIterable is closed",
    }
}

/// Operational stderr line for [`crate::output::print_log_warning`] (not session `who` tee).
#[must_use]
pub(crate) fn operational_iterable_closed_log_line(msg: &str) -> Option<&'static str> {
    let text = msg.to_ascii_lowercase();
    if text.contains("writableiterable is closed") {
        Some("acp: WritableIterable is closed")
    } else if text.contains("readableiterable is closed") {
        Some("acp: ReadableIterable is closed")
    } else {
        None
    }
}

/// Line or parent coalesced stream carries iterable-closed (split emissions included).
#[must_use]
pub(crate) fn operational_iterable_closed_for_emit(
    line: &str,
    stream_iterable_closed: Option<IterableClosedStream>,
) -> Option<&'static str> {
    if let Some(line) = operational_iterable_closed_log_line(line) {
        return Some(line);
    }
    stream_iterable_closed.map(iterable_closed_stream_message)
}

#[derive(Debug)]
pub(crate) enum AgentRetryOutcome {
    StopRetrying,
    Sleep(std::time::Duration),
}

/// Blacklist-default retry policy for bounded ACP attempts: upgrade-plan and cannot-use-model
/// errors fail fast with [`Err`]; all other errors retry with 1s then 3s sleeps until
/// [`MAX_AGENT_ATTEMPTS`]. Unknown permanent failures may spend ~4s extra before stopping.
pub(crate) fn plan_agent_retry(last_error: &str, attempt: u32) -> Result<AgentRetryOutcome, AgentError> {
    if agent_string_is_upgrade_plan(last_error) || agent_string_is_cannot_use_model(last_error) {
        return Err(AgentError(last_error.to_string()));
    }
    if attempt >= MAX_AGENT_ATTEMPTS {
        return Ok(AgentRetryOutcome::StopRetrying);
    }
    let secs = if attempt == 1 { 1_u64 } else { 3_u64 };
    Ok(AgentRetryOutcome::Sleep(std::time::Duration::from_secs(secs)))
}
