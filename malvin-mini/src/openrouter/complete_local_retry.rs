use crate::error::OpenRouterError;
use crate::openrouter::http_exchange::CompletionWithMeta;
use crate::openrouter::types::ChatMessage;
use super::super::memory_format::parse_history_response;
use super::complete_prompt_shape::{
    is_short_form_marker_turn, marker_response_missing_label, mutate_messages_after_marker_miss,
    mutate_messages_after_missing_content,
};
use super::complete_prompt_shrink::shrink_prompt_messages;
use super::complete_act_detect::{
    artifact_act_lacks_following_observation, history_has_any_artifact_act,
    history_has_exterior_without_artifact_act, latest_observation_has_nonzero_exit,
    response_has_act_fence,
};
use super::complete_act_inputs::latest_observation_has_zero_exit;
use super::complete_fail_epoch::{
    inject_exterior_before_act_cue, inject_fail_epoch_act_cue, inject_probe_after_act_cue,
    inject_unpaid_silence_act_cue,
};
use super::complete_section_shape::inject_section_shape_nudge;

pub(crate) struct LocalRetryBudget {
    pub shrink_passes: u32,
    pub missing_shape_passes: u32,
    pub marker_miss_passes: u32,
    pub fail_epoch_passes: u32,
    pub transport_stall_passes: u32,
    pub section_shape_passes: u32,
    pub max_shrink: u32,
    pub max_missing: u32,
    pub max_marker: u32,
    pub max_fail_epoch: u32,
    pub max_transport_stall: u32,
    pub max_section_shape: u32,
}

pub(crate) fn maybe_retry_local_shape(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    try_shrink_on_overflow(outcome, working, budget)
        || try_mutate_on_missing(outcome, working, budget)
        || try_mutate_on_transport_stall(outcome, working, budget)
        || try_mutate_on_marker_miss(outcome, working, budget)
        || try_section_shape_retry(outcome, working, budget)
        || try_act_pressure_retry(outcome, working, budget)
        || try_unpaid_zero_fence_as_missing(outcome, working, budget)
}

fn try_unpaid_zero_fence_as_missing(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let Ok(response) = outcome.result.as_ref() else {
        return false;
    };
    if budget.fail_epoch_passes < budget.max_fail_epoch
        || budget.missing_shape_passes >= budget.max_missing
        || is_short_form_marker_turn(working)
        || response_has_act_fence(response.content.as_str())
        || !serious_unpaid_debt(working, response.content.as_str())
    {
        return false;
    }
    if mutate_messages_after_missing_content(working) {
        budget.missing_shape_passes += 1;
        true
    } else {
        false
    }
}

fn serious_unpaid_debt(messages: &[ChatMessage], pending: &str) -> bool {
    (latest_observation_has_nonzero_exit(messages) && !response_has_act_fence(pending))
        || history_has_exterior_without_artifact_act(messages, Some(pending))
        || (artifact_act_lacks_following_observation(messages) && !response_has_act_fence(pending))
}

fn try_shrink_on_overflow(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let overflow = outcome
        .result
        .as_ref()
        .err()
        .is_some_and(OpenRouterError::is_context_overflow);
    if overflow && budget.shrink_passes < budget.max_shrink && shrink_prompt_messages(working) {
        budget.shrink_passes += 1;
        true
    } else {
        false
    }
}

fn try_mutate_on_missing(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let missing = outcome
        .result
        .as_ref()
        .err()
        .is_some_and(|e| matches!(e, OpenRouterError::MissingContent));
    if missing
        && budget.missing_shape_passes < budget.max_missing
        && mutate_messages_after_missing_content(working)
    {
        budget.missing_shape_passes += 1;
        true
    } else {
        false
    }
}

/// Request timeouts / transport stalls often mean thought-only generation burned the
/// wall-clock. Nudge toward an Act and retry locally before surfacing to the gate.
fn try_mutate_on_transport_stall(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let stalled = outcome.result.as_ref().err().is_some_and(|e| {
        matches!(e, OpenRouterError::Transport(_))
            || matches!(e, OpenRouterError::ProviderTransport { .. })
    });
    if stalled
        && budget.transport_stall_passes < budget.max_transport_stall
        && !is_short_form_marker_turn(working)
        && mutate_messages_after_missing_content(working)
    {
        budget.transport_stall_passes += 1;
        true
    } else {
        false
    }
}

fn try_mutate_on_marker_miss(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let Ok(response) = outcome.result.as_ref() else {
        return false;
    };
    if budget.marker_miss_passes < budget.max_marker
        && marker_response_missing_label(working, &response.content)
        && mutate_messages_after_marker_miss(working)
    {
        budget.marker_miss_passes += 1;
        true
    } else {
        false
    }
}

fn try_act_pressure_retry(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let Ok(response) = outcome.result.as_ref() else {
        return false;
    };
    let content = response.content.as_str();
    // Guard early: exhausted budget, marker turn, unparseable wire, or no unpaid pressure.
    // Do not drown section-shape recovery with Act cues on unparseable wire turns.
    // Green observation + no unpaid debt: allow fence-less advance (avoid review Act thrash).
    if budget.fail_epoch_passes >= budget.max_fail_epoch
        || is_short_form_marker_turn(working)
        || parse_history_response(content).is_err()
        || (latest_observation_has_zero_exit(working) && !serious_unpaid_debt(working, content))
        || !needs_act_pressure(working, content)
    {
        return false;
    }
    if inject_act_pressure_cue(working, content) {
        budget.fail_epoch_passes += 1;
        return true;
    }
    // Cues already present under serious unpaid debt: mark pressure exhausted so
    // MissingContent-shaped recovery can run in the same maybe_retry pass.
    if serious_unpaid_debt(working, content) {
        budget.fail_epoch_passes = budget.max_fail_epoch;
    }
    false
}

fn try_section_shape_retry(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    let Ok(response) = outcome.result.as_ref() else {
        return false;
    };
    if budget.section_shape_passes >= budget.max_section_shape
        || is_short_form_marker_turn(working)
        || parse_history_response(response.content.as_str()).is_ok()
    {
        return false;
    }
    if inject_section_shape_nudge(working) {
        budget.section_shape_passes += 1;
        true
    } else {
        false
    }
}

fn needs_act_pressure(messages: &[ChatMessage], content: &str) -> bool {
    serious_unpaid_debt(messages, content)
        || (!history_has_any_artifact_act(messages) && !response_has_act_fence(content))
}

fn inject_act_pressure_cue(messages: &mut Vec<ChatMessage>, content: &str) -> bool {
    if latest_observation_has_nonzero_exit(messages) && !response_has_act_fence(content) {
        return inject_fail_epoch_act_cue(messages);
    }
    if history_has_exterior_without_artifact_act(messages, Some(content)) {
        return inject_exterior_before_act_cue(messages);
    }
    if artifact_act_lacks_following_observation(messages) && !response_has_act_fence(content) {
        return inject_probe_after_act_cue(messages);
    }
    if !history_has_any_artifact_act(messages) && !response_has_act_fence(content) {
        return inject_unpaid_silence_act_cue(messages);
    }
    false
}

#[cfg(test)]
#[path = "complete_local_retry_tests.rs"]
mod complete_local_retry_tests;

#[cfg(test)]
#[path = "complete_local_retry_act_tests.rs"]
mod complete_local_retry_act_tests;
