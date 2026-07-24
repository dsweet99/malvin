use crate::openrouter::types::{ChatMessage, ChatRole};

pub(super) fn inject_fail_epoch_act_cue(messages: &mut Vec<ChatMessage>) -> bool {
    const CUE: &str = "Null Study under a failed live probe. Emit an Act fence that revises \
the named working artifact into that probe's acceptance region, then re-run the same probe. \
Do not continue or close while that probe is red.";
    inject_system_cue_or_nudge(
        messages,
        "Null Study under a failed live probe",
        CUE,
    )
}

pub(super) fn inject_unpaid_silence_act_cue(messages: &mut Vec<ChatMessage>) -> bool {
    if act_nudge_present(messages) {
        return false;
    }
    const CUE: &str = "Freeze capital is unpaid silence. Emit an Act fence that revises the \
named working artifact or runs a request-named probe in that context. Do not close.";
    inject_system_cue_or_nudge(messages, "Freeze capital is unpaid silence", CUE)
}

pub(super) fn inject_probe_after_act_cue(messages: &mut Vec<ChatMessage>) -> bool {
    if act_nudge_present(messages) {
        return false;
    }
    const CUE: &str = "Null Study: the named working artifact was revised without a following \
request-named probe observation that postdates that revision. A private probe that asserts \
the written reading does not pay. Emit an Act fence that runs a request-named probe in that \
context, then Study the recorded outcome. Do not close on an invented green.";
    inject_system_cue_or_nudge(messages, "revised without a following", CUE)
}

pub(super) fn inject_exterior_before_act_cue(messages: &mut Vec<ChatMessage>) -> bool {
    if act_nudge_present(messages) {
        return false;
    }
    const CUE: &str = "Null Study: exterior contact before revising the named working artifact. \
Emit an Act fence that revises the named working artifact (or runs a request-named probe in \
that context). Exterior Observe is unpaid silence until that Act exists.";
    inject_system_cue_or_nudge(messages, "exterior contact before revising", CUE)
}

fn inject_system_cue_or_nudge(
    messages: &mut Vec<ChatMessage>,
    already_marker: &str,
    cue: &str,
) -> bool {
    if messages
        .iter()
        .any(|m| matches!(m.role, ChatRole::System) && m.content.contains(already_marker))
    {
        return append_act_nudge(messages);
    }
    messages.insert(
        0,
        ChatMessage {
            role: ChatRole::System,
            content: cue.to_string(),
        },
    );
    true
}

fn act_nudge_present(messages: &[ChatMessage]) -> bool {
    messages.iter().any(|m| {
        matches!(m.role, ChatRole::User) && m.content.contains("Emit an Act fence now that revises")
    })
}

fn append_act_nudge(messages: &mut Vec<ChatMessage>) -> bool {
    const NUDGE: &str = "Emit an Act fence now that revises the named working artifact or \
runs a request-named probe. Exterior Observe and closing reports are null Study.";
    if act_nudge_present(messages) {
        return false;
    }
    messages.push(ChatMessage {
        role: ChatRole::User,
        content: NUDGE.to_string(),
    });
    true
}
