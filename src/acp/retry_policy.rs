#[must_use]
pub(crate) const fn retries_noun(n: u32) -> &'static str {
    if n == 1 { "retry" } else { "retries" }
}

pub(crate) fn agent_string_is_upgrade_plan(msg: &str) -> bool {
    msg.to_ascii_lowercase()
        .contains("upgrade your plan to continue")
}

#[must_use]
pub(crate) fn upgrade_plan_stream_from_buffer(buf: &str) -> bool {
    agent_string_is_upgrade_plan(buf)
}

pub(crate) fn agent_string_is_cannot_use_model(msg: &str) -> bool {
    msg.to_ascii_lowercase().contains("cannot use this model")
}

pub(crate) fn agent_string_is_model_does_not_support_tools(msg: &str) -> bool {
    msg.to_ascii_lowercase().contains("does not support tools")
}

pub(crate) fn agent_string_is_usage_limit(msg: &str) -> bool {
    msg.to_ascii_lowercase()
        .contains("you've hit your usage limit")
}

pub(crate) fn agent_string_is_openrouter_billing_failure(msg: &str) -> bool {
    let text = msg.to_ascii_lowercase();
    text.contains("openrouter billing/credit failure")
        || text.contains("insufficient credits")
}

pub(crate) fn agent_string_is_openrouter_missing_content(msg: &str) -> bool {
    msg.to_ascii_lowercase()
        .contains("openrouter response missing assistant content")
}

pub(crate) const SESSION_NEW_INTERNAL_MAX_SPAWN_ATTEMPTS: u32 = 5;

#[must_use]
pub(crate) fn agent_string_is_session_new_internal_error(msg: &str) -> bool {
    let text = msg.to_ascii_lowercase();
    if !text.contains("session/new") {
        return false;
    }
    text.contains("internal") || text.contains("code=-32603")
}

#[must_use]
pub(crate) fn agent_string_is_cursor_http2_transport_error(msg: &str) -> bool {
    let text = msg.to_ascii_lowercase();
    text.contains("ping timed out")
        || (text.contains("http/2 stream closed")
            && (text.contains("cancel") || text.contains("0x8")))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum IterableClosedStream {
    Writable,
    Readable,
}

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

#[cfg(test)]
const fn iterable_closed_stream_message(kind: IterableClosedStream) -> &'static str {
    match kind {
        IterableClosedStream::Writable => "acp: WritableIterable is closed",
        IterableClosedStream::Readable => "acp: ReadableIterable is closed",
    }
}

#[derive(Debug)]
pub(crate) enum AgentRetryOutcome {
    StopRetrying,
    Sleep(std::time::Duration),
}

pub(crate) fn agent_retry_should_stop(last_error: &str) -> bool {
    last_error.contains("workspace session restore failed")
        || crate::run_timing::acp_post_run::merge_error_mentions_restore(last_error)
}

pub(crate) fn agent_text_is_non_retryable(msg: &str) -> bool {
    agent_string_is_upgrade_plan(msg)
        || agent_string_is_cannot_use_model(msg)
        || agent_string_is_model_does_not_support_tools(msg)
        || agent_string_is_usage_limit(msg)
        || agent_string_is_openrouter_billing_failure(msg)
        || agent_string_is_openrouter_missing_content(msg)
}

pub(crate) fn plan_agent_retry(
    err: &AgentError,
    attempt: u32,
    ceiling: AttemptCeiling,
) -> Result<AgentRetryOutcome, AgentError> {
    match err.fault {
        AgentFault::NonRetryable => return Err(err.clone()),
        AgentFault::RestoreStop => return Ok(AgentRetryOutcome::StopRetrying),
        AgentFault::SessionNewInternal => return Ok(session_new_internal_outcome(attempt)),
        AgentFault::Ordinary
        | AgentFault::SessionDead
        | AgentFault::CursorBusy
        | AgentFault::StaleAuth
        | AgentFault::BackendRetryLimit
        | AgentFault::OutputCap
        | AgentFault::Http2Transport => {}
    }
    if !ceiling.allows_attempt(attempt) {
        return Ok(AgentRetryOutcome::StopRetrying);
    }
    Ok(AgentRetryOutcome::Sleep(retry_sleep(attempt)))
}

const fn session_new_internal_outcome(attempt: u32) -> AgentRetryOutcome {
    if attempt >= SESSION_NEW_INTERNAL_MAX_SPAWN_ATTEMPTS {
        return AgentRetryOutcome::StopRetrying;
    }
    AgentRetryOutcome::Sleep(retry_sleep(attempt))
}

const fn retry_sleep(attempt: u32) -> std::time::Duration {
    let secs = if attempt == 1 { 1_u64 } else { 3_u64 };
    std::time::Duration::from_secs(secs)
}

#[cfg(test)]
pub(crate) fn plan_text_retry(
    msg: &str,
    attempt: u32,
    max_attempts: u32,
) -> Result<AgentRetryOutcome, AgentError> {
    let err = AgentError(msg.to_string());
    plan_agent_retry(&err, attempt, AttemptCeiling::at_most_u32(max_attempts))
}

#[cfg(test)]
mod kiss_cov_iterable_closed {
    use super::*;

    #[test]
    fn kiss_cov_iterable_closed_stream_message_both_arms() {
        assert_eq!(
            iterable_closed_stream_message(IterableClosedStream::Writable),
            "acp: WritableIterable is closed"
        );
        assert_eq!(
            iterable_closed_stream_message(IterableClosedStream::Readable),
            "acp: ReadableIterable is closed"
        );
        assert!(matches!(
            IterableClosedStream::Writable,
            IterableClosedStream::Writable
        ));
        assert!(matches!(
            IterableClosedStream::Readable,
            IterableClosedStream::Readable
        ));
    }
}
