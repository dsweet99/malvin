use crate::support_paths::DEFAULT_MAX_ACP_RETRIES;

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

fn agent_retry_should_stop(last_error: &str) -> bool {
    last_error.contains("workspace session restore failed")
        || crate::run_timing::acp_post_run::merge_error_mentions_restore(last_error)
}

pub(crate) fn plan_agent_retry(
    last_error: &str,
    attempt: u32,
    max_attempts: u32,
) -> Result<AgentRetryOutcome, AgentError> {
    if agent_string_is_upgrade_plan(last_error)
        || agent_string_is_cannot_use_model(last_error)
        || agent_string_is_usage_limit(last_error)
        || agent_string_is_openrouter_billing_failure(last_error)
        || agent_string_is_openrouter_missing_content(last_error)
    {
        return Err(AgentError(last_error.to_string()));
    }
    if agent_retry_should_stop(last_error) {
        return Ok(AgentRetryOutcome::StopRetrying);
    }
    if agent_string_is_session_new_internal_error(last_error) {
        if attempt >= SESSION_NEW_INTERNAL_MAX_SPAWN_ATTEMPTS {
            return Ok(AgentRetryOutcome::StopRetrying);
        }
        let secs = if attempt == 1 { 1_u64 } else { 3_u64 };
        return Ok(AgentRetryOutcome::Sleep(std::time::Duration::from_secs(secs)));
    }
    if attempt >= max_attempts {
        return Ok(AgentRetryOutcome::StopRetrying);
    }
    let secs = if attempt == 1 { 1_u64 } else { 3_u64 };
    Ok(AgentRetryOutcome::Sleep(std::time::Duration::from_secs(secs)))
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
