#[must_use]
pub(crate) fn agent_string_is_stale_cursor_sdk_auth(msg: &str) -> bool {
    let text = msg.to_ascii_lowercase();
    if text.contains("authenticationerror") || text == "authentication" {
        return true;
    }
    if text.starts_with("authentication") {
        return true;
    }
    if text.contains("error_not_logged_in") {
        return true;
    }
    if text.contains("[unauthenticated]") || text.contains("unauthenticated") {
        return true;
    }
    text.contains("logged in") && text.contains("logging out")
}

#[must_use]
pub(crate) fn agent_string_is_cursor_agent_busy(msg: &str) -> bool {
    msg.to_ascii_lowercase().contains("already has active run")
}

const CHILD_OR_BRIDGE_DEAD_NEEDLES: &[&str] = &[
    "acp child process appears hung",
    "acp child process is not running",
    "acp child process is zombie",
    "acp stdout closed",
    "bridge stdout closed",
    "bridge write:",
    "bridge flush:",
    "bridge read:",
    "bridge drain timed out",
    "npm pi stdout closed",
    "npm pi write:",
    "npm pi flush:",
    "npm pi read:",
    "npm pi jsonl parse:",
    "codex stdout closed",
    "codex write:",
    "codex flush:",
    "codex read:",
    "codex json-rpc parse:",
    "currently streaming",
    "iterable is closed",
    "connection stalled",
];

fn text_has_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| text.contains(n))
}

fn text_has_drain_idle_prefix(text: &str) -> bool {
    crate::model_id::ModelBackend::ALL
        .iter()
        .any(|b| text.contains(b.drain_idle_prefix()))
}

fn retry_stop_fault(msg: &str) -> Option<crate::acp::AgentFault> {
    use crate::acp::AgentFault;
    if crate::acp::agent_text_is_non_retryable(msg) {
        return Some(AgentFault::NonRetryable);
    }
    if crate::acp::agent_retry_should_stop(msg) {
        return Some(AgentFault::RestoreStop);
    }
    if crate::acp::agent_string_is_session_new_internal_error(msg) {
        return Some(AgentFault::SessionNewInternal);
    }
    None
}

fn transport_fault(msg: &str) -> crate::acp::AgentFault {
    use crate::acp::AgentFault;
    let text = msg.to_ascii_lowercase();
    if text_has_any(&text, CHILD_OR_BRIDGE_DEAD_NEEDLES) || text_has_drain_idle_prefix(&text) {
        return AgentFault::SessionDead;
    }
    if agent_string_is_cursor_agent_busy(msg) {
        return AgentFault::CursorBusy;
    }
    if agent_string_is_stale_cursor_sdk_auth(msg) {
        return AgentFault::StaleAuth;
    }
    if crate::acp::agent_string_is_cursor_http2_transport_error(msg) {
        return AgentFault::Http2Transport;
    }
    AgentFault::Ordinary
}

#[must_use]
pub(crate) fn classify_agent_fault(msg: &str) -> crate::acp::AgentFault {
    if let Some(fault) = retry_stop_fault(msg) {
        return fault;
    }
    transport_fault(msg)
}

#[cfg(test)]
#[must_use]
pub(crate) fn agent_error_requires_coder_session_teardown(msg: &str) -> bool {
    crate::acp::AgentError(msg.to_string()).requires_coder_session_teardown()
}
