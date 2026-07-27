//! Structured assembly inputs for Study/Act cue selection (no durable transcript).

use crate::openrouter::types::{ChatMessage, ChatRole};

/// Nonzero exit on the New request (last non-nudge User).
pub(crate) fn latest_observation_has_nonzero_exit(messages: &[ChatMessage]) -> bool {
    new_request_text(messages).is_some_and(observation_reports_nonzero_exit)
}

/// Exit code 0 on the New request (last non-nudge User).
pub(crate) fn latest_observation_has_zero_exit(messages: &[ChatMessage]) -> bool {
    new_request_text(messages).is_some_and(observation_reports_zero_exit)
}

pub(crate) fn observation_reports_zero_exit(content: &str) -> bool {
    let mut saw_exit = false;
    for line in content.lines() {
        let Some(rest) = line.trim().strip_prefix("Exit code ") else {
            continue;
        };
        saw_exit = true;
        let code: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect();
        if let Ok(n) = code.parse::<i32>()
            && n != 0
        {
            return false;
        }
    }
    saw_exit
}

/// Last User message content that is not a local Act nudge.
pub(crate) fn new_request_text(messages: &[ChatMessage]) -> Option<&str> {
    messages.iter().rev().find_map(|m| {
        if !matches!(m.role, ChatRole::User) {
            return None;
        }
        if m.content.contains("Emit an Act fence now that revises") {
            return None;
        }
        Some(m.content.as_str())
    })
}

/// Previous response body: last Assistant before the New-request User.
pub(crate) fn previous_response_text(messages: &[ChatMessage]) -> Option<&str> {
    let last_user = messages.iter().rposition(|m| {
        matches!(m.role, ChatRole::User) && !m.content.contains("Emit an Act fence now that revises")
    })?;
    messages[..last_user]
        .iter()
        .rev()
        .find_map(|m| matches!(m.role, ChatRole::Assistant).then_some(m.content.as_str()))
        .or_else(|| {
            messages
                .iter()
                .rev()
                .find_map(|m| matches!(m.role, ChatRole::Assistant).then_some(m.content.as_str()))
        })
}

pub(crate) fn observation_reports_nonzero_exit(content: &str) -> bool {
    for line in content.lines() {
        let Some(rest) = line.trim().strip_prefix("Exit code ") else {
            continue;
        };
        let code: String = rest
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect();
        if let Ok(n) = code.parse::<i32>()
            && n != 0
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
#[path = "complete_act_inputs_tests.rs"]
mod complete_act_inputs_tests;
