use crate::error::OpenRouterError;
use crate::openrouter::http_exchange::CompletionWithMeta;
use crate::openrouter::types::ChatMessage;
use super::{
    is_short_form_marker_turn, marker_response_missing_label, mutate_messages_after_marker_miss,
    mutate_messages_after_missing_content, shrink_prompt_messages,
};
use super::complete_act_detect::{
    artifact_act_lacks_following_observation, history_has_any_artifact_act,
    history_has_exterior_without_artifact_act, latest_observation_has_nonzero_exit,
    response_has_act_fence,
};
use super::complete_fail_epoch::{
    inject_exterior_before_act_cue, inject_fail_epoch_act_cue, inject_probe_after_act_cue,
    inject_unpaid_silence_act_cue,
};

pub(crate) struct LocalRetryBudget {
    pub shrink_passes: u32,
    pub missing_shape_passes: u32,
    pub marker_miss_passes: u32,
    pub fail_epoch_passes: u32,
    pub max_shrink: u32,
    pub max_missing: u32,
    pub max_marker: u32,
    pub max_fail_epoch: u32,
}

pub(crate) fn maybe_retry_local_shape(
    outcome: &CompletionWithMeta,
    working: &mut Vec<ChatMessage>,
    budget: &mut LocalRetryBudget,
) -> bool {
    try_shrink_on_overflow(outcome, working, budget)
        || try_mutate_on_missing(outcome, working, budget)
        || try_mutate_on_marker_miss(outcome, working, budget)
        || try_act_pressure_retry(outcome, working, budget)
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
    if budget.fail_epoch_passes >= budget.max_fail_epoch || is_short_form_marker_turn(working) {
        return false;
    }
    let content = response.content.as_str();
    let injected = if latest_observation_has_nonzero_exit(working)
        && !response_has_act_fence(content)
    {
        inject_fail_epoch_act_cue(working)
    } else if history_has_exterior_without_artifact_act(working, Some(content)) {
        inject_exterior_before_act_cue(working)
    } else if artifact_act_lacks_following_observation(working)
        && !response_has_act_fence(content)
    {
        inject_probe_after_act_cue(working)
    } else if !history_has_any_artifact_act(working) && !response_has_act_fence(content) {
        inject_unpaid_silence_act_cue(working)
    } else {
        false
    };
    if injected {
        budget.fail_epoch_passes += 1;
    }
    injected
}
