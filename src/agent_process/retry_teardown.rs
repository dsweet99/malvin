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
    "bridge drain timed out",
    "currently streaming",
    "iterable is closed",
    "connection stalled",
];

fn text_has_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|n| text.contains(n))
}

fn text_has_backend_dead_marker(text: &str) -> bool {
    crate::config::model_id::ModelBackend::ALL.iter().any(|b| {
        let spec = b.spec();
        text.contains(spec.drain_idle_prefix())
            || spec
                .dead_session_markers()
                .iter()
                .any(|marker| text.contains(marker.as_str()))
    })
}

fn retry_stop_fault(msg: &str) -> Option<crate::agent_process::AgentFault> {
    use crate::agent_process::AgentFault;
    if crate::agent_process::agent_text_is_non_retryable(msg) {
        return Some(AgentFault::NonRetryable);
    }
    if crate::agent_process::agent_retry_should_stop(msg) {
        return Some(AgentFault::RestoreStop);
    }
    if crate::agent_process::agent_string_is_session_new_internal_error(msg) {
        return Some(AgentFault::SessionNewInternal);
    }
    None
}

fn transport_fault(msg: &str) -> crate::agent_process::AgentFault {
    use crate::agent_process::AgentFault;
    let text = msg.to_ascii_lowercase();
    if text_has_any(&text, CHILD_OR_BRIDGE_DEAD_NEEDLES) || text_has_backend_dead_marker(&text) {
        return AgentFault::SessionDead;
    }
    if agent_string_is_cursor_agent_busy(msg) {
        return AgentFault::CursorBusy;
    }
    if agent_string_is_stale_cursor_sdk_auth(msg) {
        return AgentFault::StaleAuth;
    }
    if crate::agent_process::agent_string_is_cursor_http2_transport_error(msg) {
        return AgentFault::Http2Transport;
    }
    AgentFault::Ordinary
}

#[must_use]
pub(crate) fn classify_agent_fault(msg: &str) -> crate::agent_process::AgentFault {
    if let Some(fault) = retry_stop_fault(msg) {
        return fault;
    }
    transport_fault(msg)
}

#[cfg(test)]
#[must_use]
pub(crate) fn agent_error_requires_coder_session_teardown(msg: &str) -> bool {
    crate::agent_process::AgentError(msg.to_string()).requires_coder_session_teardown()
}
