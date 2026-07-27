use super::super::types::{ChatMessage, ChatRole};

pub(super) fn is_short_form_marker_turn(messages: &[ChatMessage]) -> bool {
    messages
        .iter()
        .any(|m| matches!(m.role, ChatRole::User) && looks_like_marker_prompt(&m.content))
}

pub(super) fn looks_like_marker_prompt(content: &str) -> bool {
    (content.contains("COMPLEXITY_SCORE") || content.contains("CODING_TASK"))
        && content.contains("Pause")
}

pub(super) fn marker_response_missing_label(messages: &[ChatMessage], content: &str) -> bool {
    let Some(prompt) = messages.iter().rev().find_map(|m| {
        (matches!(m.role, ChatRole::User) && looks_like_marker_prompt(&m.content))
            .then_some(m.content.as_str())
    }) else {
        return false;
    };
    if prompt.contains("COMPLEXITY_SCORE") && !content.contains("COMPLEXITY_SCORE") {
        return true;
    }
    if prompt.contains("CODING_TASK") && !content.contains("CODING_TASK") {
        return true;
    }
    let trimmed = content.trim();
    if trimmed.is_empty() || trimmed == "Pause" || trimmed == "Pause." {
        return true;
    }
    // Marker turns forbid code fences around the label line.
    content.contains("```")
}

pub(super) fn mutate_messages_after_marker_miss(messages: &mut Vec<ChatMessage>) -> bool {
    const STERILE: &str = "Emit only the required marker line. No other text before Pause.";
    let Some(prompt) = messages.iter().rev().find_map(|m| {
        (matches!(m.role, ChatRole::User) && looks_like_marker_prompt(&m.content))
            .then_some(m.content.as_str())
    }) else {
        return false;
    };
    // Already on the minimal marker prompt — no further local shape left.
    if prompt.starts_with("Output exactly one") {
        return false;
    }
    let minimal = if prompt.contains("COMPLEXITY_SCORE") {
        "Output exactly one line then Pause:\n\nCOMPLEXITY_SCORE: <1-10>\n\nPause."
    } else if prompt.contains("CODING_TASK") {
        "Output exactly one of the two marker lines then Pause:\n\nCODING_TASK: YES\n\nor\n\nCODING_TASK: NO\n\nPause."
    } else {
        return false;
    };
    *messages = vec![
        ChatMessage {
            role: ChatRole::System,
            content: STERILE.to_string(),
        },
        ChatMessage {
            role: ChatRole::User,
            content: minimal.to_string(),
        },
    ];
    true
}
