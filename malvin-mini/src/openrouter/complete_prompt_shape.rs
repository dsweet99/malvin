use super::super::types::{ChatMessage, ChatRole};

#[path = "complete_prompt_shrink.rs"]
mod complete_prompt_shrink;
#[path = "complete_act_detect.rs"]
mod complete_act_detect;
#[path = "complete_fail_epoch.rs"]
mod complete_fail_epoch;
#[path = "complete_local_retry.rs"]
mod complete_local_retry;

pub(crate) use complete_prompt_shrink::shrink_prompt_messages;
pub(crate) use complete_local_retry::{maybe_retry_local_shape, LocalRetryBudget};
use complete_act_detect::{
    history_has_exterior_without_artifact_act, latest_observation_has_nonzero_exit,
};

const STUDY_REMINDER: &str = "State the problem and rival readings, then act with a short \
targeted trial grounded in the named working context. Study the outcome against a prior \
prediction. Freeze capital is only recorded outcomes of request-named probes you ran \
this session on the current artifact. Unrun request-named probes are unpaid silence; a \
private probe that asserts the written reading does not pay them. Empty freeze capital \
licenses only Acts that revise the named working artifact or run a request-named probe; \
exterior Observe before that Act is null Study. After a named-working-artifact revision, \
only a request-named probe outcome that postdates that revision pays freeze capital; other \
post-revision Observe is null Study. After a live probe fails, the only licensed next step \
is an Act that revises the artifact into that probe's acceptance region, then re-runs it; \
exterior Observe and closing reports are null Study. Demotion of a live fail is unmet. \
When freeze capital is green, emit the closing report and halt.";

const FAIL_EPOCH_CUE: &str = "A nonzero exit is a failed live probe. The only licensed next \
step is an Act that revises the named working artifact into that probe's acceptance \
region, then re-runs the same probe. Exterior Observe and closing reports are null Study \
until that probe is green.";

const EXTERIOR_BEFORE_ACT_CUE: &str = "Exterior contact before revising the named working \
artifact is null Study. Emit an Act that revises that artifact or runs a request-named \
probe in that context before any further exterior Observe.";

/// Short domain-agnostic Study reminder (not for marker turns).
/// Prefer fail-epoch after a red observation; else block exterior-before-Act.
pub(super) fn with_tool_use_system_reminder(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    if messages.first().is_some_and(|m| matches!(m.role, ChatRole::System))
        || is_short_form_marker_turn(messages)
    {
        return messages.to_vec();
    }
    let reminder = if latest_observation_has_nonzero_exit(messages) {
        FAIL_EPOCH_CUE
    } else if history_has_exterior_without_artifact_act(messages, None) {
        EXTERIOR_BEFORE_ACT_CUE
    } else {
        STUDY_REMINDER
    };
    let mut out = Vec::with_capacity(messages.len() + 1);
    out.push(ChatMessage {
        role: ChatRole::System,
        content: reminder.to_string(),
    });
    out.extend_from_slice(messages);
    out
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
}

fn inject_thought_only_progress_cue(messages: &mut Vec<ChatMessage>) -> bool {
    const CUE: &str = "Thought-only responses are non-progress. Emit observable content \
and a short targeted trial grounded in the named working context, then Study.";
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
                || m.content.contains("nonzero exit is a failed live probe")
                || m.content.contains("Exterior contact before revising"))
            && (m.content.contains("rival readings")
                || m.content.contains("acceptance region")
                || m.content.contains("request-named probe"))
    }) else {
        return false;
    };
    messages.remove(idx);
    true
}

#[cfg(test)]
#[path = "complete_prompt_shape_tests.rs"]
mod complete_prompt_shape_tests;
