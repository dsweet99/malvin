use super::super::types::{ChatMessage, ChatRole};
use super::complete_act_detect::{
    history_has_exterior_without_artifact_act, latest_observation_has_nonzero_exit,
};
use super::complete_act_inputs::latest_observation_has_zero_exit;
use super::complete_prompt_shrink::shrink_prompt_messages;
pub(super) use super::complete_marker_shape::{
    is_short_form_marker_turn, marker_response_missing_label, mutate_messages_after_marker_miss,
};

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

/// After a green live observation, do not demand another Act; allow fence-less advance/close.
const GREEN_OBSERVATION_CUE: &str = "The latest live observation exited 0. Do not emit another \
Act fence unless a named check is still unpaid. Prefer a fence-less reply that advances or closes.";

/// Requirements-listing turns: write string-schema JSON and pause; do not explore/fix product code.
const REQUIREMENTS_ONLY_CUE: &str = "This New request is requirements-listing only. Write \
review_requirements.json using the schema where each requirement is a plain string (not an \
object with id/description). Do not explore or edit product source. After a successful \
write and schema probe, Pause with a fence-less reply.";

/// Residual-plan / gap-analysis turns: plan in chat; do not implement product changes.
const PLAN_ONLY_CUE: &str = "This New request is gap-analysis / residual planning only. Write \
the residual plan into the chat. Do not edit product files. Prefer a fence-less reply once \
the plan is written.";

/// Short domain-agnostic Study reminder (not for marker turns).
/// Prefer fail-epoch after a red New-request observation; else block exterior-before-Act
/// from Previous response. After a green observation, inject a no-extra-Act cue so the
/// session can leave review/Act loops. When a sticky Header System already leads, inject
/// the cue as the next System message so Header + cue coexist.
pub(super) fn with_tool_use_system_reminder(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    if is_short_form_marker_turn(messages) {
        return messages.to_vec();
    }
    if cue_already_present(messages) {
        return messages.to_vec();
    }
    let Some(reminder) = select_study_act_cue(messages) else {
        return messages.to_vec();
    };
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
                || m.content.contains("Look hard for unmet evidence first")
                || m.content.contains("latest live observation exited 0")
                || m.content.contains("requirements-listing only")
                || m.content.contains("gap-analysis / residual planning only"))
    })
}

pub(super) fn select_study_act_cue(messages: &[ChatMessage]) -> Option<&'static str> {
    if latest_observation_has_nonzero_exit(messages) {
        return Some(FAIL_EPOCH_CUE);
    }
    if history_has_exterior_without_artifact_act(messages, None) {
        return Some(EXTERIOR_BEFORE_ACT_CUE);
    }
    if latest_observation_has_zero_exit(messages) {
        return Some(GREEN_OBSERVATION_CUE);
    }
    if new_request_is_requirements_only(messages) {
        return Some(REQUIREMENTS_ONLY_CUE);
    }
    if new_request_is_plan_only(messages) {
        return Some(PLAN_ONLY_CUE);
    }
    Some(STUDY_REMINDER)
}

fn new_request_is_requirements_only(messages: &[ChatMessage]) -> bool {
    use super::complete_act_inputs::new_request_text;
    new_request_text(messages).is_some_and(|t| {
        t.contains("review_requirements")
            && (t.contains("Do not start implementing")
                || t.contains("output nothing else of substance")
                || t.contains("Write **only** the JSON")
                || t.contains("Write only the JSON"))
    })
}

fn new_request_is_plan_only(messages: &[ChatMessage]) -> bool {
    use super::complete_act_inputs::new_request_text;
    new_request_text(messages).is_some_and(|t| {
        t.contains("Do not edit product files in this turn")
            || (t.contains("residual plan") && t.contains("Do not implement"))
    })
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

fn strip_injected_study_reminder(messages: &mut Vec<ChatMessage>) -> bool {
    let Some(idx) = messages.iter().position(|m| {
        matches!(m.role, ChatRole::System)
            && (m.content.contains("request-named")
                || m.content.contains("request-derived")
                || m.content.contains("nonzero exit is a failed live")
                || m.content.contains("Exterior contact before revising")
                || m.content.contains("Look hard for unmet evidence")
                || m.content.contains("latest live observation exited 0")
                || m.content.contains("requirements-listing only")
                || m.content.contains("gap-analysis / residual planning only"))
            && (m.content.contains("rival readings")
                || m.content.contains("acceptance region")
                || m.content.contains("request-named")
                || m.content.contains("Freeze capital is")
                || m.content.contains("private asserts")
                || m.content.contains("fence-less reply")
                || m.content.contains("plain string")
                || m.content.contains("Do not edit product files"))
    }) else {
        return false;
    };
    messages.remove(idx);
    true
}

#[cfg(test)]
#[path = "complete_prompt_shape_tests.rs"]
mod complete_prompt_shape_tests;
