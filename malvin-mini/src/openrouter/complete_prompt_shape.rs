use super::super::types::{ChatMessage, ChatRole};
use super::complete_act_detect::{
    history_has_exterior_without_artifact_act, latest_observation_has_nonzero_exit,
};
use super::complete_prompt_shrink::shrink_prompt_messages;

const STUDY_REMINDER: &str = "Look hard for unmet evidence first. Form a prediction, Act with a \
short targeted trial in the named working context, then Study the outcome. Only recorded \
outcomes of request-named checks on the current artifact pay; private asserts do not. After a \
named-working-artifact revision, only a post-revision request-named check pays. When \
independence from sealed work is required, regenerate live name-bindings and isolation-assay. \
When request-named checks and hard-constraint exhibits are green, emit the closing report and \
halt.";

const FAIL_EPOCH_CUE: &str = "A nonzero exit is a failed live check. Trap, poison, or \
non-binding stories about that check are unlicensed. Do not invent outcomes of checks you \
have not run. Predict that the same check will turn green, Act only into that check's \
acceptance region, then re-run it. Exterior Observe and closing reports are null Study \
until that check is green.";

const EXTERIOR_BEFORE_ACT_CUE: &str = "Exterior contact before revising the named working \
artifact is null Study. Emit an Act that revises that artifact or runs a request-named \
check in that context before any further exterior Observe.";

/// Short domain-agnostic Study reminder (not for marker turns).
/// Prefer fail-epoch after a red New-request observation; else block exterior-before-Act
/// from Previous response. When a sticky Header System already leads, inject the cue as
/// the next System message so Header + cue coexist.
pub(super) fn with_tool_use_system_reminder(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    if is_short_form_marker_turn(messages) {
        return messages.to_vec();
    }
    if cue_already_present(messages) {
        return messages.to_vec();
    }
    let reminder = select_study_act_cue(messages);
    let mut out = Vec::with_capacity(messages.len() + 1);
    if messages.first().is_some_and(|m| matches!(m.role, ChatRole::System)) {
        out.push(messages[0].clone());
        out.push(ChatMessage {
            role: ChatRole::System,
            content: reminder.to_string(),
        });
        out.extend_from_slice(&messages[1..]);
    } else {
        out.push(ChatMessage {
            role: ChatRole::System,
            content: reminder.to_string(),
        });
        out.extend_from_slice(messages);
    }
    out
}

fn cue_already_present(messages: &[ChatMessage]) -> bool {
    messages.iter().any(|m| {
        matches!(m.role, ChatRole::System)
            && (m.content.contains("nonzero exit is a failed live check")
                || m.content.contains("Exterior contact before revising")
                || m.content.contains("Look hard for unmet evidence first"))
    })
}

pub(super) fn select_study_act_cue(messages: &[ChatMessage]) -> &'static str {
    if latest_observation_has_nonzero_exit(messages) {
        FAIL_EPOCH_CUE
    } else if history_has_exterior_without_artifact_act(messages, None) {
        EXTERIOR_BEFORE_ACT_CUE
    } else {
        STUDY_REMINDER
    }
}

pub(super) fn is_short_form_marker_turn(messages: &[ChatMessage]) -> bool {
    messages
        .iter()
        .any(|m| matches!(m.role, ChatRole::User) && looks_like_marker_prompt(&m.content))
}

pub(super) fn looks_like_marker_prompt(content: &str) -> bool {
    (content.contains("COMPLEXITY_SCORE") || content.contains("CODING_TASK"))
        && content.contains("Pause")
}

pub(super) fn mutate_messages_after_missing_content(messages: &mut Vec<ChatMessage>) -> bool {
    inject_thought_only_progress_cue(messages)
        || strip_injected_study_reminder(messages)
        || shrink_prompt_messages(messages)
        || append_act_nudge_user(messages)
}

fn inject_thought_only_progress_cue(messages: &mut Vec<ChatMessage>) -> bool {
    const CUE: &str = "Thought-only responses are non-progress. Emit observable content \
and a short targeted Act fence grounded in the named working context, then Study.";
    if messages.iter().any(|m| {
        matches!(m.role, ChatRole::System) && m.content.contains("Thought-only responses")
    }) {
        return false;
    }
    messages.insert(
        0,
        ChatMessage {
            role: ChatRole::System,
            content: CUE.to_string(),
        },
    );
    true
}

fn append_act_nudge_user(messages: &mut Vec<ChatMessage>) -> bool {
    const NUDGE: &str = "Emit an Act fence now that revises the named working artifact or \
runs a request-named probe. Exterior Observe and closing reports are null Study.";
    if messages.iter().any(|m| {
        matches!(m.role, ChatRole::User) && m.content.contains("Emit an Act fence now that revises")
    }) {
        return false;
    }
    messages.push(ChatMessage {
        role: ChatRole::User,
        content: NUDGE.to_string(),
    });
    true
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

fn strip_injected_study_reminder(messages: &mut Vec<ChatMessage>) -> bool {
    let Some(idx) = messages.iter().position(|m| {
        matches!(m.role, ChatRole::System)
            && (m.content.contains("request-named")
                || m.content.contains("request-derived")
                || m.content.contains("nonzero exit is a failed live")
                || m.content.contains("Exterior contact before revising")
                || m.content.contains("Look hard for unmet evidence"))
            && (m.content.contains("rival readings")
                || m.content.contains("acceptance region")
                || m.content.contains("request-named")
                || m.content.contains("Freeze capital is")
                || m.content.contains("private asserts"))
    }) else {
        return false;
    };
    messages.remove(idx);
    true
}

#[cfg(test)]
#[path = "complete_prompt_shape_tests.rs"]
mod complete_prompt_shape_tests;
